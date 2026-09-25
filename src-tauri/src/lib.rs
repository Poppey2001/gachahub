mod download_manager;
mod games;
mod install;
mod launcher;
mod mods;
mod mod_manager;
mod platform;
mod postinstall;
mod runners;
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
async fn resolve_download_plan(game_id: String) -> Result<providers::ProviderDownloadPlan, String> {
    download_manager::resolve_plan(&game_id).await
}

#[tauri::command]
async fn start_game_download(app: tauri::AppHandle, game_id: String, install_path: String) -> Result<InstallJob, String> {
    download_manager::start(app, game_id, install_path).await
}

#[tauri::command]
fn pause_download(app: tauri::AppHandle, job_id: String, paused: bool) -> Result<download_manager::DownloadControlState, String> {
    download_manager::pause(&app, &job_id, paused)
}

#[tauri::command]
fn cancel_download(app: tauri::AppHandle, job_id: String) -> Result<download_manager::DownloadControlState, String> {
    download_manager::cancel(&app, &job_id)
}

#[tauri::command]
fn detect_runners() -> Vec<runners::RunnerInfo> {
    runners::detect()
}

#[tauri::command]
fn runner_catalog() -> Vec<runners::RunnerCatalogEntry> {
    runners::catalog()
}

#[tauri::command]
fn runner_families() -> Vec<runners::RunnerFamily> {
    runners::families()
}

#[tauri::command]
async fn runner_versions(family_id: String) -> Result<Vec<runners::RunnerVersion>, String> {
    runners::versions(&family_id).await
}

#[tauri::command]
async fn install_runner(family_id: String, tag_name: String) -> Result<runners::RunnerInfo, String> {
    runners::install(&family_id, &tag_name).await
}

#[tauri::command]
fn remove_runner(path: String) -> Result<(), String> {
    runners::remove(&path)
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


#[tauri::command]
async fn xxmi_manager_status(install_path: String, custom_launcher_path: String, wine_executable: String, wine_prefix: String) -> Result<xxmi::XxmiManagerStatus, String> {
    xxmi::manager_status(&install_path, &custom_launcher_path, &wine_executable, &wine_prefix).await
}

#[tauri::command]
async fn install_xxmi(app: tauri::AppHandle, install_path: String) -> Result<xxmi::XxmiManagerStatus, String> {
    xxmi::install_managed(app, &install_path).await
}

#[tauri::command]
async fn repair_xxmi(app: tauri::AppHandle, install_path: String) -> Result<xxmi::XxmiManagerStatus, String> {
    xxmi::repair_managed(app, &install_path).await
}

#[tauri::command]
async fn remove_xxmi(install_path: String) -> Result<(), String> {
    xxmi::remove_managed(&install_path).await
}

#[tauri::command]
fn launch_managed_xxmi(install_path: String, custom_launcher_path: String, importer: String, wine_executable: String, wine_prefix: String, no_gui: bool) -> Result<xxmi::XxmiStatus, String> {
    xxmi::managed_launch(&install_path, &custom_launcher_path, &importer, &wine_executable, &wine_prefix, no_gui)
}


#[tauri::command]
fn launch_game(
    game_id: String,
    executable_path: String,
    runtime: String,
    runner_version: String,
    runner_path: String,
    prefix_path: String,
    launch_arguments: String,
    environment_variables: std::collections::HashMap<String, String>,
    gamescope_enabled: bool,
    game_mode_enabled: bool,
    mango_hud_enabled: bool,
    prevent_sleep: bool,
) -> Result<launcher::LaunchResult, String> {
    launcher::launch(
        &game_id,
        &executable_path,
        &runtime,
        &runner_version,
        &runner_path,
        &prefix_path,
        &launch_arguments,
        environment_variables,
        gamescope_enabled,
        game_mode_enabled,
        mango_hud_enabled,
        prevent_sleep,
    )
}

#[tauri::command]
fn post_install_defaults(game_id: String, install_path: String, prefix_path: String) -> Result<postinstall::PostInstallResult, String> {
    postinstall::defaults(&game_id, &install_path, &prefix_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![list_games, platform_name, start_install, resolve_download_plan, start_game_download, pause_download, cancel_download, detect_runners, runner_catalog, runner_families, runner_versions, install_runner, remove_runner, mod_policy, scan_mods, enable_mod, adopt_mod, disable_mod, import_mod_folder, import_mod_source, archive_tool_status, sync_mod_profile, read_legacy_gmm_config, xxmi_status, launch_xxmi, xxmi_manager_status, install_xxmi, repair_xxmi, remove_xxmi, launch_managed_xxmi, launch_game, post_install_defaults])
        .run(tauri::generate_context!())
        .expect("error while running GachaHub");
}
