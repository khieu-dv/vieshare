use crate::arguments::handle_arguments;
use std::env;

mod arguments;
mod bin;
mod bootstrap;
mod commands;
mod logger;
mod utils;
use crate::commands::frps::{init_frps_config_state, init_frps_processes};
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    handle_arguments();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(init_frps_processes()) // Only manage once
        .manage(init_frps_config_state())
        .setup(|app| {
            let handle = app.handle().clone();

            // Setup cleanup on window close
            let main_window = handle.get_webview_window("main").unwrap();
            main_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { .. } = event {
                    // Clean up frpc processes before closing
                    let handle_clone = handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = commands::frps::frps_cleanup().await {
                            eprintln!("Error during cleanup: {}", e);
                        }
                    });
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::utils::util_launch_url,
            commands::bootstrap::bootstrap_check,
            commands::bootstrap::bootstrap_install,
            commands::utils::generate_app_id,
            // FRPS commands
            commands::frps::frps_connect,
            commands::frps::frps_disconnect,
            commands::frps::frps_add_port_mapping,
            commands::frps::frps_remove_port_mapping,
            commands::frps::frps_get_status,
            commands::frps::frps_test_connection,
            commands::frps::frps_load_config,
            commands::frps::frps_get_mappings,
            // frps_get_port_limits
            commands::frps::frps_get_port_limits,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
