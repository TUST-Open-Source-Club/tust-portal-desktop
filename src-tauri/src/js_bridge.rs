use tauri::{AppHandle, Manager, State};

#[cfg(target_os = "windows")]
use crate::auto_start;
use crate::log_system::LogEntry;
use crate::network_info::{self, NetworkStatus};
use crate::sign_in;
use crate::store::credentials::{self, Credentials, read_credentials};
use crate::store::settings::{self, Settings};
use crate::tray;
use crate::AppState;

// -- Login --

#[tauri::command]
pub(crate) async fn try_login(
    username: String,
    password: String,
    network_type: String,
    state: State<'_, AppState>,
) -> Result<sign_in::LoginResult, String> {
    sign_in::try_login(&state.http_client, username, password, network_type).await
}

// -- Network status --

#[tauri::command]
pub(crate) fn check_network_status() -> NetworkStatus {
    network_info::check_network_status()
}

// -- Credentials --

#[tauri::command]
pub(crate) fn save_credentials(
    username: String,
    password: String,
    network_type: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let creds = Credentials {
        username,
        password,
        network_type,
    };
    let json = serde_json::to_string(&creds).map_err(|e| e.to_string())?;
    let path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?
        .join(credentials::CREDS_FILE);
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    tracing::info!(frontend = true, message = "凭据已保存");
    Ok(())
}

#[tauri::command]
pub(crate) fn load_credentials(app_handle: AppHandle) -> Result<Option<Credentials>, String> {
    Ok(read_credentials(&app_handle))
}

// -- Settings --

#[tauri::command]
pub(crate) fn load_settings(app_handle: AppHandle) -> Result<Option<Settings>, String> {
    Ok(settings::read_settings(&app_handle))
}

// -- Logs --

#[tauri::command]
pub(crate) fn get_logs(state: State<'_, AppState>) -> Vec<LogEntry> {
    state.logs.lock().unwrap().clone()
}

// -- Auto-login pause --

#[tauri::command]
pub(crate) fn get_auto_login_paused(state: State<'_, AppState>) -> bool {
    *state.auto_login_paused.lock().unwrap()
}

#[tauri::command]
pub(crate) fn set_auto_login_paused(
    paused: bool,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) {
    *state.auto_login_paused.lock().unwrap() = paused;
    tray::rebuild_tray_menu(&app_handle, paused);
    let msg = if paused {
        "自动登录已暂停"
    } else {
        "自动登录已恢复"
    };
    tracing::info!(frontend = true, message = msg);
}

// -- Ignore SSID --

#[tauri::command]
pub(crate) fn get_ignore_ssid(state: State<'_, AppState>) -> bool {
    *state.ignore_ssid.lock().unwrap()
}

#[tauri::command]
pub(crate) fn set_ignore_ssid(ignore: bool, app_handle: AppHandle, state: State<'_, AppState>) {
    *state.ignore_ssid.lock().unwrap() = ignore;
    settings::save_ignore_ssid(&app_handle, ignore).ok();
}

// -- Platform --

#[tauri::command]
pub(crate) fn get_platform() -> String {
    get_platform_impl()
}

#[cfg(target_os = "windows")]
fn get_platform_impl() -> String {
    "windows".to_string()
}

#[cfg(target_os = "macos")]
fn get_platform_impl() -> String {
    "macos".to_string()
}

// -- Auto-Start (Windows only) --

#[tauri::command]
pub(crate) fn get_auto_start_enabled() -> Result<bool, String> {
    get_auto_start_enabled_impl()
}

#[cfg(target_os = "windows")]
fn get_auto_start_enabled_impl() -> Result<bool, String> {
    Ok(auto_start::is_enabled())
}

#[cfg(not(target_os = "windows"))]
fn get_auto_start_enabled_impl() -> Result<bool, String> {
    Err("Auto-start is only supported on Windows".to_string())
}

#[tauri::command]
pub(crate) fn set_auto_start_enabled(enabled: bool) -> Result<(), String> {
    set_auto_start_enabled_impl(enabled)
}

#[cfg(target_os = "windows")]
fn set_auto_start_enabled_impl(enabled: bool) -> Result<(), String> {
    auto_start::set_enabled(enabled)
}

#[cfg(not(target_os = "windows"))]
fn set_auto_start_enabled_impl(_enabled: bool) -> Result<(), String> {
    Err("Auto-start is only supported on Windows".to_string())
}
