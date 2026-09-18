use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs,
    io,
    path::{Component, Path, PathBuf},
    process::Command,
};

const REGISTRY_FILE: &str = ".gachahub-managed.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ManagedRegistry {
    entries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModEntry {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub source_path: String,
    pub active_path: String,
    pub enabled: bool,
    pub managed: bool,
    pub status: String,
    pub ini_count: u64,
    pub size_bytes: u64,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub conflicts: usize,
    pub broken: usize,
    pub managed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModSnapshot {
    pub game_id: String,
    pub library_path: String,
    pub active_path: String,
    pub entries: Vec<ModEntry>,
    pub stats: ModStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LegacyGmmConfig {
    pub config_path: String,
    pub library_path: Option<String>,
    pub active_mods_path: Option<String>,
    pub source: String,
}

fn expand_user(raw: &str) -> PathBuf {
    let raw = raw.trim();
    if raw == "~" || raw.starts_with("~/") || raw.starts_with("~\\") {
        let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"));
        if let Some(home) = home {
            if raw.len() == 1 {
                return PathBuf::from(home);
            }
            return PathBuf::from(home).join(&raw[2..]);
        }
    }
    PathBuf::from(raw)
}

fn safe_relative(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err("invalid mod id".into()),
        }
    }
    Ok(path)
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn directory_metrics(path: &Path) -> (u64, u64) {
    fn walk(path: &Path, size: &mut u64, ini: &mut u64) {
        let Ok(read) = fs::read_dir(path) else { return };
        for entry in read.flatten() {
            let p = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                walk(&p, size, ini);
            } else if ft.is_file() {
                if let Ok(meta) = entry.metadata() { *size = size.saturating_add(meta.len()); }
                if p.extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("ini")).unwrap_or(false) {
                    *ini += 1;
                }
            }
        }
    }
    let mut size = 0;
    let mut ini = 0;
    walk(path, &mut size, &mut ini);
    (size, ini)
}

fn contains_direct_ini(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry.file_type().map(|t| t.is_file()).unwrap_or(false)
                && entry.path().extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("ini")).unwrap_or(false)
        })
}

fn collect_mod_dirs(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, path: &Path, out: &mut Vec<PathBuf>, depth: usize) {
        if depth > 6 { return; }
        if contains_direct_ini(path) {
            out.push(path.to_path_buf());
            return;
        }
        let Ok(read) = fs::read_dir(path) else { return };
        let mut child_dirs = Vec::new();
        let mut has_files = false;
        for entry in read.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() { child_dirs.push(entry.path()); }
                else if ft.is_file() { has_files = true; }
            }
        }
        if child_dirs.is_empty() && has_files && path != root {
            out.push(path.to_path_buf());
            return;
        }
        for child in child_dirs { walk(root, &child, out, depth + 1); }
    }

    let mut out = Vec::new();
    if root.exists() { walk(root, root, &mut out, 0); }
    out
}

fn registry_path(active: &Path) -> PathBuf { active.join(REGISTRY_FILE) }

fn read_registry(active: &Path) -> ManagedRegistry {
    fs::read_to_string(registry_path(active))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_registry(active: &Path, registry: &ManagedRegistry) -> Result<(), String> {
    fs::create_dir_all(active).map_err(|e| e.to_string())?;
    let data = serde_json::to_vec_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(registry_path(active), data).map_err(|e| e.to_string())
}

fn snapshot(game_id: &str, library_raw: &str, active_raw: &str) -> Result<ModSnapshot, String> {
    let library = expand_user(library_raw);
    fs::create_dir_all(&library).map_err(|e| format!("cannot create mod library: {e}"))?;
    let active = if active_raw.trim().is_empty() { PathBuf::new() } else { expand_user(active_raw) };
    if !active.as_os_str().is_empty() {
        fs::create_dir_all(&active).map_err(|e| format!("cannot create active mods path: {e}"))?;
    }

    let registry = if active.as_os_str().is_empty() { ManagedRegistry::default() } else { read_registry(&active) };
    let requires_ini = matches!(game_id, "genshin" | "hsr" | "zzz" | "wuwa" | "endfield");
    let candidates = collect_mod_dirs(&library);
    let mut basename_count: HashMap<String, usize> = HashMap::new();
    for source in &candidates {
        if let Some(name) = source.file_name().and_then(|x| x.to_str()) {
            *basename_count.entry(name.to_lowercase()).or_default() += 1;
        }
    }

    let mut entries = Vec::new();
    for source in candidates {
        let relative = source.strip_prefix(&library).unwrap_or(&source).to_string_lossy().replace('\\', "/");
        let name = source.file_name().and_then(|x| x.to_str()).unwrap_or("Unnamed Mod").to_string();
        let destination = if active.as_os_str().is_empty() { PathBuf::new() } else { active.join(&name) };
        let (size_bytes, ini_count) = directory_metrics(&source);
        let managed_source = registry.entries.get(&name);
        let managed = managed_source.map(|x| x == &relative).unwrap_or(false);
        let enabled = !destination.as_os_str().is_empty() && destination.exists();
        let duplicate = basename_count.get(&name.to_lowercase()).copied().unwrap_or(0) > 1;

        let (status, conflict_reason) = if duplicate {
            ("conflict".to_string(), Some("Mehrere Library-Mods haben denselben Zielordnernamen.".into()))
        } else if enabled && !managed {
            ("conflict".to_string(), Some("Im aktiven Mods-Ordner existiert bereits ein nicht von GachaHub verwalteter Ordner mit diesem Namen.".into()))
        } else if requires_ini && ini_count == 0 {
            ("broken".to_string(), Some("Für diesen XXMI/3DMigoto-Adapter wurde keine INI-Datei im Mod gefunden.".into()))
        } else if enabled {
            ("enabled".to_string(), None)
        } else {
            ("disabled".to_string(), None)
        };

        entries.push(ModEntry {
            id: relative.clone(),
            name,
            relative_path: relative,
            source_path: source.to_string_lossy().into_owned(),
            active_path: destination.to_string_lossy().into_owned(),
            enabled,
            managed,
            status,
            ini_count,
            size_bytes,
            conflict_reason,
        });
    }
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let stats = ModStats {
        total: entries.len(),
        enabled: entries.iter().filter(|m| m.status == "enabled").count(),
        disabled: entries.iter().filter(|m| m.status == "disabled").count(),
        conflicts: entries.iter().filter(|m| m.status == "conflict").count(),
        broken: entries.iter().filter(|m| m.status == "broken").count(),
        managed: entries.iter().filter(|m| m.managed).count(),
    };

    Ok(ModSnapshot {
        game_id: game_id.into(),
        library_path: library.to_string_lossy().into_owned(),
        active_path: active.to_string_lossy().into_owned(),
        entries,
        stats,
    })
}

fn deploy(source: &Path, destination: &Path, mode: &str) -> Result<(), String> {
    match mode {
        "symlink" => {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(source, destination).map_err(|e| e.to_string())?;
                Ok(())
            }
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_dir(source, destination).map_err(|e| format!("symlink failed (Developer Mode/Admin may be required): {e}"))?;
                Ok(())
            }
            #[cfg(not(any(unix, windows)))]
            { Err("symlink deployment is not supported on this platform".into()) }
        }
        _ => copy_dir_all(source, destination).map_err(|e| e.to_string()),
    }
}

pub fn scan(game_id: &str, library: &str, active: &str) -> Result<ModSnapshot, String> {
    snapshot(game_id, library, active)
}

pub fn enable(game_id: &str, mod_id: &str, library_raw: &str, active_raw: &str, mode: &str) -> Result<ModSnapshot, String> {
    if active_raw.trim().is_empty() { return Err("active mods path is not configured".into()); }
    let relative = safe_relative(mod_id)?;
    let library = expand_user(library_raw);
    let source = library.join(&relative);
    if !source.is_dir() { return Err("mod source directory does not exist".into()); }
    let name = source.file_name().and_then(|x| x.to_str()).ok_or_else(|| "invalid mod folder name".to_string())?.to_string();
    let active = expand_user(active_raw);
    fs::create_dir_all(&active).map_err(|e| e.to_string())?;
    let destination = active.join(&name);
    let mut registry = read_registry(&active);

    if destination.exists() {
        if registry.entries.get(&name).map(|x| x == mod_id).unwrap_or(false) {
            return snapshot(game_id, library_raw, active_raw);
        }
        return Err(format!("active destination already exists and is not managed by this mod: {}", destination.display()));
    }

    deploy(&source, &destination, mode)?;
    registry.entries.insert(name, mod_id.to_string());
    write_registry(&active, &registry)?;
    snapshot(game_id, library_raw, active_raw)
}

pub fn adopt(game_id: &str, mod_id: &str, library_raw: &str, active_raw: &str) -> Result<ModSnapshot, String> {
    if active_raw.trim().is_empty() { return Err("active mods path is not configured".into()); }
    let relative = safe_relative(mod_id)?;
    let library = expand_user(library_raw);
    let source = library.join(&relative);
    if !source.is_dir() { return Err("mod source directory does not exist".into()); }
    let name = source.file_name().and_then(|x| x.to_str()).ok_or_else(|| "invalid mod folder name".to_string())?.to_string();
    let active = expand_user(active_raw);
    let destination = active.join(&name);
    if !destination.exists() { return Err("there is no active folder to adopt".into()); }

    let candidates = collect_mod_dirs(&library);
    let same_name = candidates.iter().filter(|candidate| candidate.file_name() == source.file_name()).count();
    if same_name > 1 { return Err("cannot adopt because multiple library mods use the same active folder name".into()); }

    let mut registry = read_registry(&active);
    if let Some(existing) = registry.entries.get(&name) {
        if existing != mod_id { return Err("active folder is already managed by another library mod".into()); }
    }
    registry.entries.insert(name, mod_id.to_string());
    write_registry(&active, &registry)?;
    snapshot(game_id, library_raw, active_raw)
}

pub fn disable(game_id: &str, mod_id: &str, library_raw: &str, active_raw: &str) -> Result<ModSnapshot, String> {
    if active_raw.trim().is_empty() { return Err("active mods path is not configured".into()); }
    let relative = safe_relative(mod_id)?;
    let library = expand_user(library_raw);
    let source = library.join(&relative);
    let name = source.file_name().and_then(|x| x.to_str()).ok_or_else(|| "invalid mod folder name".to_string())?.to_string();
    let active = expand_user(active_raw);
    let destination = active.join(&name);
    let mut registry = read_registry(&active);

    if registry.entries.get(&name).map(|x| x == mod_id).unwrap_or(false) {
        if destination.exists() {
            let meta = fs::symlink_metadata(&destination).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() {
                #[cfg(unix)]
                fs::remove_file(&destination).map_err(|e| e.to_string())?;
                #[cfg(windows)]
                fs::remove_dir(&destination).map_err(|e| e.to_string())?;
                #[cfg(not(any(unix, windows)))]
                fs::remove_file(&destination).map_err(|e| e.to_string())?;
            } else { fs::remove_dir_all(&destination).map_err(|e| e.to_string())?; }
        }
        registry.entries.remove(&name);
        write_registry(&active, &registry)?;
        return snapshot(game_id, library_raw, active_raw);
    }

    if destination.exists() {
        return Err("refusing to remove an unmanaged active mod folder".into());
    }
    snapshot(game_id, library_raw, active_raw)
}

pub fn import_folder(game_id: &str, source_raw: &str, library_raw: &str, active_raw: &str) -> Result<ModSnapshot, String> {
    let source = expand_user(source_raw);
    if !source.is_dir() { return Err("import source must be a directory in v0.5".into()); }
    let library = expand_user(library_raw);
    fs::create_dir_all(&library).map_err(|e| e.to_string())?;
    let name = source.file_name().and_then(|x| x.to_str()).ok_or_else(|| "invalid source folder".to_string())?;
    let destination = library.join(name);
    if destination.exists() { return Err(format!("library destination already exists: {}", destination.display())); }
    copy_dir_all(&source, &destination).map_err(|e| e.to_string())?;
    snapshot(game_id, library_raw, active_raw)
}


fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("cannot open ZIP archive: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("invalid ZIP archive: {e}"))?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| format!("cannot read ZIP entry: {e}"))?;
        let Some(relative) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            return Err(format!("unsafe path in ZIP archive: {}", entry.name()));
        };
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut output = fs::File::create(&target).map_err(|e| e.to_string())?;
        io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&target, fs::Permissions::from_mode(mode));
        }
    }
    Ok(())
}

fn find_7zip() -> Option<String> {
    for candidate in ["7zz", "7z", "7za"] {
        if Command::new(candidate).arg("i").output().is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn extract_with_7zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let tool = find_7zip().ok_or_else(|| {
        "7z/7zz was not found. Install 7-Zip to import .7z and .rar archives.".to_string()
    })?;
    let output_arg = format!("-o{}", destination.to_string_lossy());
    let status = Command::new(&tool)
        .arg("x")
        .arg("-y")
        .arg(&output_arg)
        .arg(archive_path)
        .status()
        .map_err(|e| format!("failed to run {tool}: {e}"))?;
    if !status.success() {
        return Err(format!("{tool} failed to extract archive (exit: {status})"));
    }
    Ok(())
}

fn import_extracted_root(extracted: &Path, archive_path: &Path, library: &Path) -> Result<(), String> {
    let mut entries = fs::read_dir(extracted)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    if entries.is_empty() {
        return Err("archive is empty".into());
    }

    // Preserve a clean single-root archive. Otherwise group the archive content under its archive name.
    if entries.len() == 1 && entries[0].file_type().map(|t| t.is_dir()).unwrap_or(false) {
        let source = entries[0].path();
        let name = source.file_name().ok_or_else(|| "invalid archive root folder".to_string())?;
        let destination = library.join(name);
        if destination.exists() {
            return Err(format!("library destination already exists: {}", destination.display()));
        }
        copy_dir_all(&source, &destination).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let stem = archive_path
        .file_stem()
        .and_then(|x| x.to_str())
        .filter(|x| !x.trim().is_empty())
        .unwrap_or("Imported Mod");
    let destination = library.join(stem);
    if destination.exists() {
        return Err(format!("library destination already exists: {}", destination.display()));
    }
    copy_dir_all(extracted, &destination).map_err(|e| e.to_string())
}

pub fn import_archive(game_id: &str, source_raw: &str, library_raw: &str, active_raw: &str) -> Result<ModSnapshot, String> {
    let source = expand_user(source_raw);
    if !source.is_file() {
        return Err("archive source does not exist".into());
    }
    let extension = source.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(extension.as_str(), "zip" | "7z" | "rar") {
        return Err("supported archives are .zip, .7z and .rar".into());
    }

    let library = expand_user(library_raw);
    fs::create_dir_all(&library).map_err(|e| e.to_string())?;
    let temp = env::temp_dir().join(format!("gachahub-mod-import-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp).map_err(|e| e.to_string())?;

    let result = (|| {
        match extension.as_str() {
            "zip" => extract_zip(&source, &temp)?,
            "7z" | "rar" => extract_with_7zip(&source, &temp)?,
            _ => unreachable!(),
        }
        import_extracted_root(&temp, &source, &library)
    })();

    let _ = fs::remove_dir_all(&temp);
    result?;
    snapshot(game_id, library_raw, active_raw)
}

pub fn import_source(game_id: &str, source_raw: &str, library_raw: &str, active_raw: &str) -> Result<ModSnapshot, String> {
    let source = expand_user(source_raw);
    if source.is_dir() {
        return import_folder(game_id, source_raw, library_raw, active_raw);
    }
    if source.is_file() {
        return import_archive(game_id, source_raw, library_raw, active_raw);
    }
    Err("import source does not exist".into())
}

pub fn archive_tool_status() -> String {
    find_7zip().unwrap_or_else(|| "not-found".into())
}

pub fn sync_profile(game_id: &str, enabled_ids: &[String], library_raw: &str, active_raw: &str, mode: &str) -> Result<ModSnapshot, String> {
    let initial = snapshot(game_id, library_raw, active_raw)?;
    let desired: std::collections::HashSet<&str> = enabled_ids.iter().map(|x| x.as_str()).collect();

    for entry in initial.entries.iter().filter(|e| e.managed && e.enabled && !desired.contains(e.id.as_str())) {
        disable(game_id, &entry.id, library_raw, active_raw)?;
    }
    let current = snapshot(game_id, library_raw, active_raw)?;
    for entry in current.entries.iter().filter(|e| !e.enabled && e.status == "disabled" && desired.contains(e.id.as_str())) {
        enable(game_id, &entry.id, library_raw, active_raw, mode)?;
    }
    snapshot(game_id, library_raw, active_raw)
}

fn find_string_by_keys(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let normalized = key.to_lowercase().replace('_', "").replace('-', "").replace(' ', "");
                if keys.iter().any(|candidate| normalized == *candidate) {
                    if let Some(s) = val.as_str() { if !s.trim().is_empty() { return Some(s.to_string()); } }
                }
            }
            for val in map.values() {
                if let Some(found) = find_string_by_keys(val, keys) { return Some(found); }
            }
            None
        }
        serde_json::Value::Array(items) => items.iter().find_map(|v| find_string_by_keys(v, keys)),
        _ => None,
    }
}

pub fn read_legacy_config(raw_path: &str) -> Result<LegacyGmmConfig, String> {
    let mut path = expand_user(raw_path);
    if path.is_dir() {
        for candidate in ["config.json", "settings.json", "appsettings.json"] {
            let p = path.join(candidate);
            if p.exists() { path = p; break; }
        }
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("cannot read legacy GMM config: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| format!("invalid JSON config: {e}"))?;
    let library_path = find_string_by_keys(&value, &["librarypath", "modlibrarypath", "modslibrarypath", "libraryroot"]);
    let active_mods_path = find_string_by_keys(&value, &["activemodspath", "modspath", "activepath", "xxmimodspath", "genshinmodspath"]);
    Ok(LegacyGmmConfig {
        config_path: path.to_string_lossy().into_owned(),
        library_path,
        active_mods_path,
        source: "legacy-gmm-json".into(),
    })
}
