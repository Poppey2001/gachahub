use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::Command,
};
use tar::Archive;
use uuid::Uuid;
use xz2::read::XzDecoder;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerInfo {
    pub id: String,
    pub name: String,
    pub family: String,
    pub kind: String,
    pub version: String,
    pub path: String,
    pub source: String,
    pub installed: bool,
    pub managed_by_umu: bool,
    pub removable: bool,
    pub recommended_backend: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerCatalogEntry {
    pub id: String,
    pub name: String,
    pub family: String,
    pub kind: String,
    pub source: String,
    pub recommended_backend: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerFamily {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub provider: String,
    pub source: String,
    pub installable: bool,
    pub recommended_backend: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerVersion {
    pub id: String,
    pub family_id: String,
    pub name: String,
    pub tag_name: String,
    pub asset_name: String,
    pub download_url: String,
    pub size: u64,
    pub published_at: String,
    pub prerelease: bool,
    pub installed: bool,
    pub installed_path: String,
}

#[derive(Debug, Clone)]
struct FamilyConfig {
    id: &'static str,
    name: &'static str,
    short_name: &'static str,
    provider: &'static str,
    source: &'static str,
    api: &'static str,
    backend: &'static str,
    description: &'static str,
    tag_prefix: Option<&'static str>,
}

fn family_configs() -> Vec<FamilyConfig> {
    vec![
        FamilyConfig { id:"ge-proton", name:"Proton (GE)", short_name:"GE", provider:"github", source:"GloriousEggroll", api:"https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases", backend:"UMU", description:"GE-Proton community builds with additional compatibility fixes.", tag_prefix:Some("GE-Proton") },
        FamilyConfig { id:"dwproton", name:"Proton (DW)", short_name:"DW", provider:"forgejo", source:"Dawn Winery", api:"https://dawn.wine/api/v1/repos/dawn-winery/dwproton/releases", backend:"UMU", description:"Dawn Winery Proton builds focused on anime and gacha game compatibility.", tag_prefix:Some("dwproton-") },
        FamilyConfig { id:"cachyos-proton", name:"Proton (CachyOS)", short_name:"CY", provider:"github", source:"CachyOS", api:"https://api.github.com/repos/CachyOS/proton-cachyos/releases", backend:"UMU", description:"CachyOS optimized Proton releases.", tag_prefix:None },
        FamilyConfig { id:"proton-em", name:"Proton (EM)", short_name:"EM", provider:"github", source:"Etaash Mathamsetty", api:"https://api.github.com/repos/Etaash-mathamsetty/Proton/releases", backend:"UMU", description:"Development-oriented Proton-EM builds with Wayland and compatibility work.", tag_prefix:Some("EM-") },
        FamilyConfig { id:"proton-sarek", name:"Proton (Sarek)", short_name:"SR", provider:"github", source:"Proton-Sarek", api:"https://api.github.com/repos/pythonlover02/Proton-Sarek/releases", backend:"UMU", description:"Community Proton builds with DXVK-Sarek variants.", tag_prefix:Some("Proton-Sarek") },
        FamilyConfig { id:"proton-wineland", name:"Proton (Wineland)", short_name:"WL", provider:"github", source:"Wineland", api:"https://api.github.com/repos/nanomatters/proton-cachyos/releases", backend:"UMU", description:"Wineland branch with native Wayland-oriented work.", tag_prefix:Some("wineland-") },
        FamilyConfig { id:"steam-proton", name:"Proton (Valve)", short_name:"VL", provider:"local", source:"Steam", api:"", backend:"Steam / UMU", description:"Valve Proton versions managed primarily by Steam.", tag_prefix:None },
        FamilyConfig { id:"umu-proton", name:"Proton (UMU)", short_name:"UM", provider:"local", source:"umu-launcher", api:"", backend:"UMU", description:"Proton runtime managed automatically by umu-launcher.", tag_prefix:None },
    ]
}

fn home() -> Option<PathBuf> {
    env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")).map(PathBuf::from)
}

fn managed_root() -> Result<PathBuf, String> {
    let h = home().ok_or_else(|| "home directory not found".to_string())?;
    Ok(h.join(".local/share/gachahub/runners"))
}

fn cache_root() -> Result<PathBuf, String> {
    let h = home().ok_or_else(|| "home directory not found".to_string())?;
    Ok(h.join(".cache/gachahub/runners"))
}

fn which(executable: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(executable);
        if candidate.is_file() { return Some(candidate); }
        #[cfg(windows)] {
            let exe = dir.join(format!("{executable}.exe"));
            if exe.is_file() { return Some(exe); }
        }
    }
    None
}

fn normalize_name(raw: &str) -> String { raw.replace('_', " ").trim().to_string() }

fn classify_proton(name: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    let lower = name.to_ascii_lowercase();
    if lower.contains("dwproton") || lower.contains("dw-proton") { ("DWProton", "dwproton", "UMU", "Dawn Winery Proton build") }
    else if lower.contains("wineland") { ("Proton-Wineland", "proton-wineland", "UMU", "Wineland Proton build") }
    else if lower.contains("cachy") { ("Proton-CachyOS", "cachyos-proton", "UMU", "CachyOS optimized Proton build") }
    else if lower.contains("proton-em") || lower.contains("proton_em") || lower.starts_with("em-") { ("Proton-EM", "proton-em", "UMU", "Community Proton-EM build") }
    else if lower.contains("sarek") { ("Proton-Sarek", "proton-sarek", "UMU", "Community Proton-Sarek build") }
    else if lower.contains("tkg") && lower.contains("proton") { ("Proton-TKG", "proton-tkg", "UMU", "Community Proton-TKG build") }
    else if lower.contains("ge-proton") || lower.contains("proton-ge") || lower.starts_with("geproton") { ("GE-Proton", "ge-proton", "UMU", "Community Proton build by GloriousEggroll") }
    else if lower.contains("experimental") { ("Proton Experimental", "proton-experimental", "Steam / UMU", "Valve experimental Proton channel") }
    else if lower.contains("hotfix") { ("Proton Hotfix", "proton-hotfix", "Steam / UMU", "Valve hotfix Proton channel") }
    else if lower.contains("umu") { ("UMU-Proton", "umu-proton", "UMU", "UMU managed Proton runtime") }
    else { ("Valve Proton", "steam-proton", "Steam / UMU", "Valve Proton compatibility tool") }
}

fn classify_wine(name: &str) -> (&'static str, &'static str, &'static str) {
    let lower = name.to_ascii_lowercase();
    if lower.contains("wine-ge") || lower.contains("wine_ge") || lower.contains("ge-wine") { ("Wine-GE", "wine-ge", "Wine") }
    else if lower.contains("lutris") { ("Lutris Wine", "lutris-wine", "Wine") }
    else { ("Wine", "wine", "Wine") }
}

fn looks_like_proton(path: &Path) -> bool {
    path.join("proton").is_file() || path.join("compatibilitytool.vdf").is_file() || path.file_name().and_then(|x| x.to_str()).map(|x| x.to_ascii_lowercase().contains("proton") || x.to_ascii_lowercase().contains("dwproton") || x.to_ascii_lowercase().contains("wineland")).unwrap_or(false)
}

fn add_proton_dir(map: &mut BTreeMap<String, RunnerInfo>, dir: &Path, source: &str) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_dir() || !looks_like_proton(&path) { continue; }
        let raw_name = entry.file_name().to_string_lossy().into_owned();
        let (family, kind, backend, notes) = classify_proton(&raw_name);
        let path_s = path.to_string_lossy().into_owned();
        let id = format!("{}:{}", kind, path_s);
        map.entry(id.clone()).or_insert(RunnerInfo { id, name:normalize_name(&raw_name), family:family.into(), kind:kind.into(), version:normalize_name(&raw_name), path:path_s, source:source.into(), installed:true, managed_by_umu:kind=="umu-proton", removable:source=="GachaHub managed", recommended_backend:backend.into(), notes:notes.into() });
    }
}

fn add_wine_dir(map: &mut BTreeMap<String, RunnerInfo>, dir: &Path, source: &str) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path(); if !path.is_dir() { continue; }
        let raw_name = entry.file_name().to_string_lossy().into_owned();
        let wine_bin = if path.join("bin/wine").is_file() { path.join("bin/wine") } else if path.join("wine").is_file() { path.join("wine") } else { continue; };
        let (family, kind, backend) = classify_wine(&raw_name);
        let path_s = wine_bin.to_string_lossy().into_owned(); let id = format!("{}:{}",kind,path_s);
        map.entry(id.clone()).or_insert(RunnerInfo { id, name:normalize_name(&raw_name), family:family.into(), kind:kind.into(), version:normalize_name(&raw_name), path:path_s, source:source.into(), installed:true, managed_by_umu:false, removable:false, recommended_backend:backend.into(), notes:"Wine runner discovered on the local system".into() });
    }
}

fn wine_version(path:&Path)->String { Command::new(path).arg("--version").output().ok().filter(|o|o.status.success()).map(|o|String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s|!s.is_empty()).unwrap_or_else(||"System Wine".into()) }

pub fn detect() -> Vec<RunnerInfo> {
    let mut found=BTreeMap::<String,RunnerInfo>::new();
    #[cfg(windows)] { found.insert("native-windows".into(), RunnerInfo{id:"native-windows".into(),name:"Windows Native".into(),family:"Native".into(),kind:"native".into(),version:"Windows".into(),path:String::new(),source:"Operating system".into(),installed:true,managed_by_umu:false,removable:false,recommended_backend:"Native".into(),notes:"Run the Windows game executable directly".into()}); }
    #[cfg(not(windows))]
    if let Some(home)=home() {
        let proton_dirs=[
            (home.join(".local/share/gachahub/runners"),"GachaHub managed"),
            (home.join(".steam/root/compatibilitytools.d"),"Steam compatibilitytools.d"),(home.join(".steam/steam/compatibilitytools.d"),"Steam compatibilitytools.d"),(home.join(".steam/steam/steamapps/common"),"Steam bundled"),(home.join(".local/share/Steam/compatibilitytools.d"),"Steam compatibilitytools.d"),(home.join(".local/share/Steam/steamapps/common"),"Steam bundled"),(home.join(".var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d"),"Steam Flatpak"),(home.join(".var/app/com.valvesoftware.Steam/data/Steam/steamapps/common"),"Steam Flatpak bundled"),(home.join(".config/heroic/tools/proton"),"Heroic"),(home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/tools/proton"),"Heroic Flatpak")];
        for (dir,source) in proton_dirs { add_proton_dir(&mut found,&dir,source); }
        let wine_dirs=[(home.join(".local/share/lutris/runners/wine"),"Lutris"),(home.join(".config/heroic/tools/wine"),"Heroic"),(home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/tools/wine"),"Heroic Flatpak"),(home.join(".local/share/bottles/runners"),"Bottles"),(home.join(".var/app/com.usebottles.bottles/data/bottles/runners"),"Bottles Flatpak")];
        for (dir,source) in wine_dirs { add_wine_dir(&mut found,&dir,source); }
    }
    #[cfg(not(windows))] {
        for dir in [PathBuf::from("/usr/share/steam/compatibilitytools.d"),PathBuf::from("/usr/local/share/steam/compatibilitytools.d")] { add_proton_dir(&mut found,&dir,"System compatibilitytools.d"); }
        if let Some(umu)=which("umu-run") { let p=umu.to_string_lossy().into_owned(); found.insert("umu-managed".into(),RunnerInfo{id:"umu-managed".into(),name:"UMU-Proton (managed)".into(),family:"UMU-Proton".into(),kind:"umu-proton".into(),version:"Automatic".into(),path:p,source:"umu-launcher".into(),installed:true,managed_by_umu:true,removable:false,recommended_backend:"UMU".into(),notes:"UMU can manage its Proton runtime automatically".into()}); }
        if let Some(wine)=which("wine") { let version=wine_version(&wine); let p=wine.to_string_lossy().into_owned(); found.insert("system-wine".into(),RunnerInfo{id:"system-wine".into(),name:version.clone(),family:"System Wine".into(),kind:"system-wine".into(),version,path:p,source:"System PATH".into(),installed:true,managed_by_umu:false,removable:false,recommended_backend:"Wine".into(),notes:"Distribution-provided Wine".into()}); }
    }
    found.into_values().collect()
}

pub fn families() -> Vec<RunnerFamily> {
    family_configs().into_iter().map(|c| RunnerFamily { id:c.id.into(), name:c.name.into(), short_name:c.short_name.into(), provider:c.provider.into(), source:c.source.into(), installable:c.provider!="local", recommended_backend:c.backend.into(), description:c.description.into() }).collect()
}

pub fn catalog() -> Vec<RunnerCatalogEntry> {
    families().into_iter().map(|f|RunnerCatalogEntry{id:f.id.clone(),name:f.name.clone(),family:"Proton".into(),kind:f.id.clone(),source:f.source.clone(),recommended_backend:f.recommended_backend.clone(),description:f.description.clone()}).collect()
}

#[derive(Debug, Deserialize)]
struct GithubAsset { name:String, browser_download_url:String, size:u64 }
#[derive(Debug, Deserialize)]
struct GithubRelease { tag_name:String, name:Option<String>, prerelease:bool, published_at:Option<String>, assets:Vec<GithubAsset> }
#[derive(Debug, Deserialize)]
struct ForgeAsset { name:String, browser_download_url:String, size:u64 }
#[derive(Debug, Deserialize)]
struct ForgeRelease { tag_name:String, name:Option<String>, prerelease:bool, published_at:Option<String>, assets:Vec<ForgeAsset> }

fn archive_score(name:&str)->i32 {
    let l=name.to_ascii_lowercase();
    if !(l.ends_with(".tar.gz")||l.ends_with(".tgz")||l.ends_with(".tar.xz")||l.ends_with(".txz")||l.ends_with(".tar.zst")||l.ends_with(".zip")) { return -10000; }
    if l.contains("sha")||l.contains("checksum")||l.contains("torrent")||l.contains("source")||l.contains("debug") { return -10000; }
    let mut score=100;
    if l.contains("x86_64")||l.contains("amd64") { score+=30; }
    if l.contains("wow64") { score-=5; }
    if l.contains("x86_64_v3")||l.contains("x86_64-v3")||l.contains("x86_64_v4")||l.contains("x86_64-v4") { score-=25; }
    if l.contains("async") { score-=8; }
    if l.contains("slr") { score+=3; }
    score
}

fn installed_match(tag:&str)->Option<String> {
    let tl=tag.to_ascii_lowercase();
    detect().into_iter().find(|r| { let v=r.version.to_ascii_lowercase(); let n=r.name.to_ascii_lowercase(); v==tl || n==tl || v.contains(&tl) || tl.contains(&v) }).map(|r|r.path)
}

async fn get_text(url:&str)->Result<String,String> {
    let client=reqwest::Client::builder().user_agent("GachaHub/0.14.1 runner-manager").build().map_err(|e|e.to_string())?;
    let response=client.get(url).header("Accept","application/vnd.github+json").send().await.map_err(|e|e.to_string())?;
    if !response.status().is_success() { return Err(format!("runner source returned HTTP {}",response.status())); }
    response.text().await.map_err(|e|e.to_string())
}

pub async fn versions(family_id:&str)->Result<Vec<RunnerVersion>,String> {
    let cfg=family_configs().into_iter().find(|c|c.id==family_id).ok_or_else(||format!("unknown runner family: {family_id}"))?;
    if cfg.provider=="local" {
        return Ok(detect().into_iter().filter(|r| match family_id { "umu-proton"=>r.kind=="umu-proton", "steam-proton"=>r.kind=="steam-proton"||r.kind=="proton-experimental"||r.kind=="proton-hotfix", _=>false }).map(|r|RunnerVersion{id:r.id.clone(),family_id:family_id.into(),name:r.version.clone(),tag_name:r.version.clone(),asset_name:String::new(),download_url:String::new(),size:0,published_at:String::new(),prerelease:false,installed:true,installed_path:r.path.clone()}).collect());
    }
    let url=if cfg.provider=="github" { format!("{}?per_page=100&page=1",cfg.api) } else { format!("{}?limit=100&page=1",cfg.api) };
    let text=get_text(&url).await?;
    let mut out=Vec::new();
    if cfg.provider=="github" {
        let releases:Vec<GithubRelease>=serde_json::from_str(&text).map_err(|e|format!("failed to parse GitHub releases: {e}"))?;
        for rel in releases {
            if let Some(prefix)=cfg.tag_prefix { if !rel.tag_name.to_ascii_lowercase().starts_with(&prefix.to_ascii_lowercase()) { continue; } }
            let Some(asset)=rel.assets.into_iter().max_by_key(|a|archive_score(&a.name)) else { continue; };
            if archive_score(&asset.name)<0 { continue; }
            let installed=installed_match(&rel.tag_name);
            out.push(RunnerVersion{id:format!("{}:{}",family_id,rel.tag_name),family_id:family_id.into(),name:rel.name.filter(|n|!n.trim().is_empty()).unwrap_or_else(||rel.tag_name.clone()),tag_name:rel.tag_name,asset_name:asset.name,download_url:asset.browser_download_url,size:asset.size,published_at:rel.published_at.unwrap_or_default(),prerelease:rel.prerelease,installed:installed.is_some(),installed_path:installed.unwrap_or_default()});
        }
    } else {
        let releases:Vec<ForgeRelease>=serde_json::from_str(&text).map_err(|e|format!("failed to parse Forgejo releases: {e}"))?;
        for rel in releases {
            if let Some(prefix)=cfg.tag_prefix { if !rel.tag_name.to_ascii_lowercase().starts_with(&prefix.to_ascii_lowercase()) { continue; } }
            let Some(asset)=rel.assets.into_iter().max_by_key(|a|archive_score(&a.name)) else { continue; };
            if archive_score(&asset.name)<0 { continue; }
            let installed=installed_match(&rel.tag_name);
            out.push(RunnerVersion{id:format!("{}:{}",family_id,rel.tag_name),family_id:family_id.into(),name:rel.name.filter(|n|!n.trim().is_empty()).unwrap_or_else(||rel.tag_name.clone()),tag_name:rel.tag_name,asset_name:asset.name,download_url:asset.browser_download_url,size:asset.size,published_at:rel.published_at.unwrap_or_default(),prerelease:rel.prerelease,installed:installed.is_some(),installed_path:installed.unwrap_or_default()});
        }
    }
    Ok(out)
}

fn safe_name(input:&str)->String { input.chars().map(|c|if c.is_ascii_alphanumeric()||matches!(c,'.'|'_'|'-'){c}else{'_'}).collect() }

fn unpack_tar<R:Read>(reader:R,dst:&Path)->Result<(),String> {
    let mut archive=Archive::new(reader);
    let entries=archive.entries().map_err(|e|e.to_string())?;
    for item in entries { let mut entry=item.map_err(|e|e.to_string())?; entry.unpack_in(dst).map_err(|e|e.to_string())?; }
    Ok(())
}

fn extract_archive(bytes:&[u8],asset:&str,dst:&Path)->Result<(),String> {
    let l=asset.to_ascii_lowercase();
    fs::create_dir_all(dst).map_err(|e|e.to_string())?;
    if l.ends_with(".tar.gz")||l.ends_with(".tgz") { return unpack_tar(GzDecoder::new(Cursor::new(bytes)),dst); }
    if l.ends_with(".tar.xz")||l.ends_with(".txz") { return unpack_tar(XzDecoder::new(Cursor::new(bytes)),dst); }
    if l.ends_with(".tar.zst") { let decoder=zstd::stream::read::Decoder::new(Cursor::new(bytes)).map_err(|e|e.to_string())?; return unpack_tar(decoder,dst); }
    if l.ends_with(".zip") {
        let mut archive=ZipArchive::new(Cursor::new(bytes)).map_err(|e|e.to_string())?;
        for i in 0..archive.len() { let mut file=archive.by_index(i).map_err(|e|e.to_string())?; let Some(rel)=file.enclosed_name().map(|p|p.to_owned()) else { continue; }; let out=dst.join(rel); if file.is_dir(){fs::create_dir_all(&out).map_err(|e|e.to_string())?;} else {if let Some(p)=out.parent(){fs::create_dir_all(p).map_err(|e|e.to_string())?;} let mut f=fs::File::create(out).map_err(|e|e.to_string())?; std::io::copy(&mut file,&mut f).map_err(|e|e.to_string())?;} }
        return Ok(());
    }
    Err(format!("unsupported runner archive: {asset}"))
}

fn find_proton_root(base:&Path,depth:usize)->Option<PathBuf> {
    if looks_like_proton(base) && (base.join("proton").is_file()||base.join("compatibilitytool.vdf").is_file()) { return Some(base.to_path_buf()); }
    if depth==0 { return None; }
    let read=fs::read_dir(base).ok()?;
    for e in read.flatten() { let p=e.path(); if p.is_dir() { if let Some(found)=find_proton_root(&p,depth-1){return Some(found);} } }
    None
}

fn copy_dir_all(src:&Path,dst:&Path)->Result<(),String> {
    fs::create_dir_all(dst).map_err(|e|e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e|e.to_string())? { let entry=entry.map_err(|e|e.to_string())?; let ty=entry.file_type().map_err(|e|e.to_string())?; let target=dst.join(entry.file_name()); if ty.is_dir(){copy_dir_all(&entry.path(),&target)?;} else {fs::copy(entry.path(),target).map_err(|e|e.to_string())?;} }
    Ok(())
}

pub async fn install(family_id:&str,tag_name:&str)->Result<RunnerInfo,String> {
    let version=versions(family_id).await?.into_iter().find(|v|v.tag_name==tag_name).ok_or_else(||format!("version not found: {tag_name}"))?;
    if version.download_url.is_empty() { return Err("this runner is managed externally".into()); }
    let client=reqwest::Client::builder().user_agent("GachaHub/0.14.1 runner-manager").build().map_err(|e|e.to_string())?;
    let response=client.get(&version.download_url).send().await.map_err(|e|e.to_string())?;
    if !response.status().is_success(){return Err(format!("download failed with HTTP {}",response.status()));}
    let bytes=response.bytes().await.map_err(|e|e.to_string())?;
    if version.size > 0 && bytes.len() as u64 != version.size {
        return Err(format!(
            "runner download size mismatch: expected {} bytes, received {} bytes",
            version.size,
            bytes.len()
        ));
    }
    let root=managed_root()?; fs::create_dir_all(&root).map_err(|e|e.to_string())?;
    let cache=cache_root()?; fs::create_dir_all(&cache).map_err(|e|e.to_string())?;
    let staging=cache.join(format!("extract-{}",Uuid::new_v4())); fs::create_dir_all(&staging).map_err(|e|e.to_string())?;
    extract_archive(&bytes,&version.asset_name,&staging)?;
    let proton_root=find_proton_root(&staging,3).ok_or_else(||"downloaded archive does not contain a Proton compatibility tool".to_string())?;
    let final_dir=root.join(safe_name(&version.tag_name)); if final_dir.exists(){fs::remove_dir_all(&final_dir).map_err(|e|e.to_string())?;}
    copy_dir_all(&proton_root,&final_dir)?;
    let _=fs::remove_dir_all(&staging);
    detect().into_iter().find(|r|Path::new(&r.path)==final_dir.as_path()).or_else(||detect().into_iter().find(|r|r.version.to_ascii_lowercase().contains(&version.tag_name.to_ascii_lowercase()))).ok_or_else(||"runner installed but could not be detected".into())
}

pub fn remove(path:&str)->Result<(),String> {
    let root=managed_root()?; let target=fs::canonicalize(path).map_err(|e|e.to_string())?; let root_canon=fs::canonicalize(&root).unwrap_or(root);
    if !target.starts_with(&root_canon){return Err("only GachaHub-managed runners can be removed here".into());}
    fs::remove_dir_all(target).map_err(|e|e.to_string())
}
