use std::path::PathBuf;

use tauri::Manager;

use crate::{
    models::{AiProviderConfig, AiScriptResult, ApkAnalysis, DeviceFileEntry, FridaRunRequest, FridaStatus, ProviderModel, RunnerSessionInfo, TerminalSessionInfo, TerminalSessionRequest},
    services::{adb, ai, apk, runner, terminal},
};

fn resource_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resource_dir()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn analyze_apk(file_path: String) -> Result<ApkAnalysis, String> {
    apk::analyze_apk(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_tmp_files(app: tauri::AppHandle, _port: u16) -> Result<Vec<DeviceFileEntry>, String> {
    let dir = resource_dir(&app)?;
    adb::list_tmp_files(&dir).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_frida_server(app: tauri::AppHandle, port: u16, binary_path: Option<String>) -> Result<FridaStatus, String> {
    let dir = resource_dir(&app)?;
    let binary = binary_path.unwrap_or_else(|| "/data/local/tmp/frida-server".to_string());
    adb::start_frida_server(&dir, port, &binary).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn stop_frida_server(app: tauri::AppHandle) -> Result<FridaStatus, String> {
    let dir = resource_dir(&app)?;
    adb::stop_frida_server(&dir).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_frida_status(app: tauri::AppHandle, port: u16) -> Result<FridaStatus, String> {
    let dir = resource_dir(&app)?;
    adb::get_frida_status(&dir, port).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn fetch_provider_models(config: AiProviderConfig) -> Result<Vec<ProviderModel>, String> {
    ai::fetch_models(&config).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn generate_frida_script(
    config: AiProviderConfig,
    package_name: String,
    so_name: Option<String>,
    hardening_info: Option<String>,
    analysis_notes: Option<Vec<String>>,
) -> Result<AiScriptResult, String> {
    ai::generate_script(
        &config,
        &package_name,
        so_name.as_deref(),
        hardening_info.as_deref(),
        analysis_notes.as_deref().unwrap_or(&[]),
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_terminal_session(
    app: tauri::AppHandle,
    state: tauri::State<terminal::TerminalState>,
    request: TerminalSessionRequest,
) -> Result<TerminalSessionInfo, String> {
    terminal::start_session(app, state, &request.mode).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn write_terminal_input(
    state: tauri::State<terminal::TerminalState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    terminal::write_session(state, &session_id, &data).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn close_terminal_session(
    state: tauri::State<terminal::TerminalState>,
    session_id: String,
) -> Result<(), String> {
    terminal::close_session(state, &session_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_frida_run(
    app: tauri::AppHandle,
    state: tauri::State<runner::RunnerState>,
    request: FridaRunRequest,
) -> Result<RunnerSessionInfo, String> {
    runner::start_frida_run(app, state, request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn stop_frida_run(
    state: tauri::State<runner::RunnerState>,
    session_id: String,
) -> Result<(), String> {
    runner::stop_run(state, &session_id).map_err(|error| error.to_string())
}
