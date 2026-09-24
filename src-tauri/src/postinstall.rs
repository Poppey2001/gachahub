use serde::Serialize;
use std::{collections::HashMap, fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

#[cfg(not(target_os = "windows"))]
const GENSHIN_LINUX: &str = include_str!("../../scripts/postinstall/genshin-linux.sh");
#[cfg(target_os = "windows")]
const GENSHIN_WINDOWS: &str = include_str!("../../scripts/postinstall/genshin-windows.ps1");

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostInstallSettings {
    pub override_enabled: bool,
    pub install_path: String,
    pub runtime: String,
    pub runner_version: String,
    pub runner_path: String,
    pub prefix_path: String,
    pub launch_arguments: String,
    pub environment_variables: String,
    pub gamescope_enabled: bool,
    pub game_mode_enabled: bool,
    pub mango_hud_enabled: bool,
    pub prevent_sleep: bool,
    pub auto_update: bool,
    pub allow_preloads: bool,
    pub update_channel: String,
    pub mods_enabled: bool,
    pub verify_before_launch: bool,
    pub xxmi_enabled: bool,
    pub post_install_profile: String,
    pub post_install_applied_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPostInstallTools {
    pub game_mode: bool,
    pub gamescope: bool,
    pub mango_hud: bool,
    pub umu: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostInstallResult {
    pub game_id: String,
    pub platform: String,
    pub executable_path: String,
    pub settings: PostInstallSettings,
    pub detected: DetectedPostInstallTools,
    pub warnings: Vec<String>,
}

fn parse_bool(value: Option<&String>) -> bool {
    matches!(value.map(String::as_str), Some("true") | Some("1") | Some("yes"))
}

fn now_stamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    secs.to_string()
}

fn parse_output(stdout: &str) -> (HashMap<String, String>, Vec<String>) {
    let mut values = HashMap::new();
    let mut warnings = Vec::new();
    for line in stdout.lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        if key == "warning" {
            warnings.push(value.to_string());
        } else {
            values.insert(key.to_string(), value.to_string());
        }
    }
    (values, warnings)
}

#[cfg(unix)]
fn run_script(script: &str, extension: &str, install_path: &str, prefix_path: &str) -> Result<String, String> {
    use std::os::unix::fs::PermissionsExt;
    let file = std::env::temp_dir().join(format!("gachahub-postinstall-{}-{}.{}", std::process::id(), now_stamp(), extension));
    fs::write(&file, script).map_err(|e| format!("post-install script write failed: {e}"))?;
    let mut perms = fs::metadata(&file).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&file, perms).map_err(|e| e.to_string())?;
    let output = Command::new("bash")
        .arg(&file)
        .arg(install_path)
        .arg(prefix_path)
        .output()
        .map_err(|e| format!("post-install script failed to start: {e}"))?;
    let _ = fs::remove_file(&file);
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(windows)]
fn run_script(script: &str, extension: &str, install_path: &str, prefix_path: &str) -> Result<String, String> {
    let file = std::env::temp_dir().join(format!("gachahub-postinstall-{}-{}.{}", std::process::id(), now_stamp(), extension));
    fs::write(&file, script).map_err(|e| format!("post-install script write failed: {e}"))?;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&file)
        .arg("-InstallPath").arg(install_path)
        .arg("-PrefixPath").arg(prefix_path)
        .output()
        .map_err(|e| format!("post-install script failed to start: {e}"))?;
    let _ = fs::remove_file(&file);
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn defaults(game_id: &str, install_path: &str, prefix_path: &str) -> Result<PostInstallResult, String> {
    if game_id != "genshin" {
        return Err(format!("no post-install profile registered for {game_id}"));
    }

    #[cfg(target_os = "windows")]
    let stdout = run_script(GENSHIN_WINDOWS, "ps1", install_path, prefix_path)?;
    #[cfg(not(target_os = "windows"))]
    let stdout = run_script(GENSHIN_LINUX, "sh", install_path, prefix_path)?;

    let (values, warnings) = parse_output(&stdout);
    let get = |key: &str| values.get(key).cloned().unwrap_or_default();

    Ok(PostInstallResult {
        game_id: "genshin".into(),
        platform: get("platform"),
        executable_path: get("executablePath"),
        settings: PostInstallSettings {
            override_enabled: true,
            install_path: get("installPath"),
            runtime: get("runtime"),
            runner_version: get("runnerVersion"),
            runner_path: get("runnerPath"),
            prefix_path: get("prefixPath"),
            launch_arguments: get("launchArguments"),
            environment_variables: get("environmentVariables"),
            gamescope_enabled: parse_bool(values.get("gamescopeEnabled")),
            game_mode_enabled: parse_bool(values.get("gameModeEnabled")),
            mango_hud_enabled: parse_bool(values.get("mangoHudEnabled")),
            prevent_sleep: parse_bool(values.get("preventSleep")),
            auto_update: parse_bool(values.get("autoUpdate")),
            allow_preloads: parse_bool(values.get("allowPreloads")),
            update_channel: get("updateChannel"),
            mods_enabled: parse_bool(values.get("modsEnabled")),
            verify_before_launch: parse_bool(values.get("verifyBeforeLaunch")),
            xxmi_enabled: parse_bool(values.get("xxmiEnabled")),
            post_install_profile: get("profileVersion"),
            post_install_applied_at: now_stamp(),
        },
        detected: DetectedPostInstallTools {
            game_mode: parse_bool(values.get("hasGameMode")),
            gamescope: parse_bool(values.get("hasGamescope")),
            mango_hud: parse_bool(values.get("hasMangoHud")),
            umu: parse_bool(values.get("hasUmu")),
        },
        warnings,
    })
}
