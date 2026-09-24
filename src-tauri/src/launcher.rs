use serde::Serialize;
use std::{collections::HashMap, path::Path, process::{Command, Stdio}};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub pid: u32,
    pub game_id: String,
    pub runtime: String,
    pub executable_path: String,
    pub prefix_path: String,
    pub runner_version: String,
    pub runner_path: String,
    pub command_preview: String,
}

fn expand_home(value: &str) -> String {
    if let Some(rest) = value.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    value.to_string()
}


fn infer_prefix_from_exe(exe: &str) -> Option<String> {
    let path = Path::new(exe);
    for ancestor in path.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("drive_c") {
            return ancestor.parent().map(|parent| parent.to_string_lossy().to_string());
        }
    }
    None
}

fn split_args(value: &str) -> Vec<String> {
    // Small shell-like parser for the launcher UI. Supports quoted arguments and backslash escapes.
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if ch == q { quote = None; } else { current.push(ch); }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if escaped { current.push('\\'); }
    if !current.is_empty() { out.push(current); }
    out
}

fn command_exists(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

#[cfg(target_os = "windows")]
pub fn launch(
    game_id: &str,
    executable_path: &str,
    _runtime: &str,
    runner_version: &str,
    runner_path: &str,
    prefix_path: &str,
    launch_arguments: &str,
    environment_variables: HashMap<String, String>,
    _gamescope_enabled: bool,
    _game_mode_enabled: bool,
    _mango_hud_enabled: bool,
    _prevent_sleep: bool,
) -> Result<LaunchResult, String> {
    let exe = expand_home(executable_path);
    if !Path::new(&exe).is_file() {
        return Err(format!("Game executable not found: {exe}"));
    }
    let mut cmd = Command::new(&exe);
    cmd.args(split_args(launch_arguments));
    for (key, value) in environment_variables { cmd.env(key, value); }
    if let Some(parent) = Path::new(&exe).parent() { cmd.current_dir(parent); }
    let child = cmd.spawn().map_err(|e| format!("failed to start game: {e}"))?;
    Ok(LaunchResult {
        pid: child.id(), game_id: game_id.into(), runtime: "native".into(), executable_path: exe,
        prefix_path: expand_home(prefix_path), runner_version: runner_version.into(),
        runner_path: expand_home(runner_path), command_preview: "native executable".into(),
    })
}

#[cfg(not(target_os = "windows"))]
pub fn launch(
    game_id: &str,
    executable_path: &str,
    runtime: &str,
    runner_version: &str,
    runner_path: &str,
    prefix_path: &str,
    launch_arguments: &str,
    environment_variables: HashMap<String, String>,
    gamescope_enabled: bool,
    game_mode_enabled: bool,
    mango_hud_enabled: bool,
    prevent_sleep: bool,
) -> Result<LaunchResult, String> {
    let exe = expand_home(executable_path);
    let prefix = infer_prefix_from_exe(&exe).unwrap_or_else(|| expand_home(prefix_path));
    if !Path::new(&exe).is_file() {
        return Err(format!("Game executable not found: {exe}"));
    }
    if prefix.trim().is_empty() {
        return Err("Wine/Proton prefix is empty. Open the game settings and select the correct prefix.".into());
    }

    let use_wine = runtime.eq_ignore_ascii_case("wine");
    let configured_runner_path = expand_home(runner_path.trim());
    let base_program = if use_wine && !configured_runner_path.is_empty() {
        configured_runner_path.clone()
    } else if use_wine {
        "wine".into()
    } else {
        "umu-run".into()
    };
    if !command_exists(&base_program) {
        return Err(if use_wine {
            format!("Wine runner was not found or could not start: {base_program}")
        } else {
            "umu-run was not found in PATH. Install umu-launcher or choose Wine as runtime.".into()
        });
    }
    if !use_wine && !configured_runner_path.is_empty() && !Path::new(&configured_runner_path).exists() {
        return Err(format!(
            "Selected Proton runner no longer exists: {configured_runner_path}. Refresh the runner list or choose Automatic."
        ));
    }

    let mut program = base_program;
    let mut args = vec![exe.clone()];
    args.extend(split_args(launch_arguments));

    if game_mode_enabled && command_exists("gamemoderun") {
        args.insert(0, program);
        program = "gamemoderun".into();
    }
    if gamescope_enabled && command_exists("gamescope") {
        let mut wrapped = vec!["--".into(), program];
        wrapped.extend(args);
        args = wrapped;
        program = "gamescope".into();
    }
    if prevent_sleep && command_exists("systemd-inhibit") {
        let mut wrapped = vec![
            "--what=sleep:idle".into(),
            "--why=GachaHub game session".into(),
            "--mode=block".into(),
            program,
        ];
        wrapped.extend(args);
        args = wrapped;
        program = "systemd-inhibit".into();
    }

    let mut cmd = Command::new(&program);
    cmd.args(&args);
    cmd.env("WINEPREFIX", &prefix);
    if !use_wine {
        // Only pass PROTONPATH for a concrete detected runner. Runner labels and
        // compatibility recommendations are display metadata, not safe executable
        // identifiers. With no path, umu-run selects/manages its own Proton runtime.
        if !configured_runner_path.is_empty() {
            cmd.env("PROTONPATH", &configured_runner_path);
        }
        cmd.env("GAMEID", if game_id == "genshin" { "umu-genshin" } else { "umu-default" });
    }
    if mango_hud_enabled { cmd.env("MANGOHUD", "1"); }
    for (key, value) in environment_variables { cmd.env(key, value); }
    if let Some(parent) = Path::new(&exe).parent() { cmd.current_dir(parent); }
    cmd.stdout(Stdio::null()).stderr(Stdio::null());

    let preview = format!("{} {}", program, args.join(" "));
    let child = cmd.spawn().map_err(|e| format!("failed to start game: {e}"))?;
    Ok(LaunchResult {
        pid: child.id(), game_id: game_id.into(), runtime: runtime.into(), executable_path: exe,
        prefix_path: prefix, runner_version: runner_version.into(),
        runner_path: configured_runner_path, command_preview: preview,
    })
}
