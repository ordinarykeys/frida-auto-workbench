mod commands;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(services::runner::RunnerState::default())
        .manage(services::terminal::TerminalState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::analyze_apk,
            commands::list_tmp_files,
            commands::start_frida_server,
            commands::stop_frida_server,
            commands::get_frida_status,
            commands::fetch_provider_models,
            commands::generate_frida_script,
            commands::start_terminal_session,
            commands::write_terminal_input,
            commands::close_terminal_session,
            commands::start_frida_run,
            commands::stop_frida_run
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
