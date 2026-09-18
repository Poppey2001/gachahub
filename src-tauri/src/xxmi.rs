use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XxmiStatus {
    pub importer: String,
    pub launcher_path: String,
    pub exists: bool,
    pub platform: String,
    pub launch_command: Vec<String>,
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

fn validate_importer(importer: &str) -> Result<String, String> {
    let importer = importer.trim().to_ascii_uppercase();
    match importer.as_str() {
        "GIMI" | "SRMI" | "ZZMI" | "WWMI" | "EFMI" | "HIMI" => Ok(importer),
        _ => Err(format!("unsupported XXMI importer: {importer}")),
    }
}

pub fn status(launcher_path: &str, importer: &str, wine_executable: &str, no_gui: bool) -> Result<XxmiStatus, String> {
    let importer = validate_importer(importer)?;
    let launcher = expand_user(launcher_path);
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unsupported"
    };

    let mut command = Vec::new();
    if cfg!(target_os = "linux") {
        command.push(if wine_executable.trim().is_empty() { "wine".into() } else { wine_executable.trim().into() });
    }
    command.push(launcher.to_string_lossy().into_owned());
    if no_gui {
        command.push("--nogui".into());
    }
    command.push("--xxmi".into());
    command.push(importer.clone());

    Ok(XxmiStatus {
        importer,
        launcher_path: launcher.to_string_lossy().into_owned(),
        exists: launcher.is_file(),
        platform: platform.into(),
        launch_command: command,
    })
}

pub fn launch(launcher_path: &str, importer: &str, wine_executable: &str, no_gui: bool) -> Result<XxmiStatus, String> {
    let status = status(launcher_path, importer, wine_executable, no_gui)?;
    if !status.exists {
        return Err(format!("XXMI Launcher not found: {}", status.launcher_path));
    }

    let launcher = PathBuf::from(&status.launcher_path);
    let mut command = if cfg!(target_os = "windows") {
        Command::new(&launcher)
    } else if cfg!(target_os = "linux") {
        let wine = if wine_executable.trim().is_empty() { "wine" } else { wine_executable.trim() };
        let mut cmd = Command::new(wine);
        cmd.arg(&launcher);
        cmd
    } else {
        return Err("XXMI launching is currently implemented for Windows and Linux/Wine only".into());
    };

    if no_gui {
        command.arg("--nogui");
    }
    command.arg("--xxmi").arg(&status.importer);

    if let Some(parent) = launcher.parent() {
        command.current_dir(parent);
    }

    command
        .spawn()
        .map_err(|e| format!("failed to launch XXMI: {e}"))?;

    Ok(status)
}
