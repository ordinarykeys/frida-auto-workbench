mod commands;

use commands::frida::FridaState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(FridaState::default())
        .invoke_handler(tauri::generate_handler![
            commands::apk::analyze_apk,
            commands::adb::adb_devices,
            commands::adb::adb_list_frida_files,
            commands::adb::adb_push_frida,
            commands::adb::adb_get_processes,
            commands::adb::adb_install,
            commands::frida::frida_start,
            commands::frida::frida_stop,
            commands::frida::frida_status,
            commands::frida::frida_forward,
            commands::frida::frida_ps,
            commands::ai::ai_list_models,
            commands::ai::ai_chat,
            commands::ai::ai_generate_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
