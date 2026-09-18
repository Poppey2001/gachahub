mod games;
mod install;
mod mods;
mod mod_manager;
mod platform;
mod providers;
mod xxmi;

use games::{Game, GameRegistry};
use install::InstallJob;

#[tauri::command]
fn list_games() -> Vec<Game> {
    GameRegistry::builtin().games
}

#[tauri::command]
fn platform_name() -> String {
    platform::current_platform().to_string()
}

#[tauri::command]
fn start_install(game_id: String) -> Result<InstallJob, String> {
    install::create_install_job(&game_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn mod_policy(game_id: String) -> Result<mods::ModPolicy, String> {
    mods::policy_for(&game_id).ok_or_else(|| format!("unknown game: {game_id}"))
}


#[tauri::command]
fn scan_mods(game_id: String, library_path: String, active_path: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::scan(&game_id, &library_path, &active_path)
}

#[tauri::command]
fn enable_mod(game_id: String, mod_id: String, library_path: String, active_path: String, deployment_mode: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::enable(&game_id, &mod_id, &library_path, &active_path, &deployment_mode)
}


#[tauri::command]
fn adopt_mod(game_id: String, mod_id: String, library_path: String, active_path: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::adopt(&game_id, &mod_id, &library_path, &active_path)
}

#[tauri::command]
fn disable_mod(game_id: String, mod_id: String, library_path: String, active_path: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::disable(&game_id, &mod_id, &library_path, &active_path)
}

#[tauri::command]
fn import_mod_folder(game_id: String, source_path: String, library_path: String, active_path: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::import_folder(&game_id, &source_path, &library_path, &active_path)
}

#[tauri::command]
fn import_mod_source(game_id: String, source_path: String, library_path: String, active_path: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::import_source(&game_id, &source_path, &library_path, &active_path)
}

#[tauri::command]
fn archive_tool_status() -> String {
    mod_manager::archive_tool_status()
}

#[tauri::command]
fn sync_mod_profile(game_id: String, enabled_ids: Vec<String>, library_path: String, active_path: String, deployment_mode: String) -> Result<mod_manager::ModSnapshot, String> {
    mod_manager::sync_profile(&game_id, &enabled_ids, &library_path, &active_path, &deployment_mode)
}

#[tauri::command]
fn read_legacy_gmm_config(path: String) -> Result<mod_manager::LegacyGmmConfig, String> {
    mod_manager::read_legacy_config(&path)
}


#[tauri::command]
fn xxmi_status(launcher_path: String, importer: String, wine_executable: String, no_gui: bool) -> Result<xxmi::XxmiStatus, String> {
    xxmi::status(&launcher_path, &importer, &wine_executable, no_gui)
}

#[tauri::command]
fn launch_xxmi(launcher_path: String, importer: String, wine_executable: String, no_gui: bool) -> Result<xxmi::XxmiStatus, String> {
    xxmi::launch(&launcher_path, &importer, &wine_executable, no_gui)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_games, platform_name, start_install, mod_policy, scan_mods, enable_mod, adopt_mod, disable_mod, import_mod_folder, import_mod_source, archive_tool_status, sync_mod_profile, read_legacy_gmm_config, xxmi_status, launch_xxmi])
        .run(tauri::generate_context!())
        .expect("error while running GachaHub");
}
