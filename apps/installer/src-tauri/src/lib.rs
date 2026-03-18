mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::env_detect::detect_env,
            commands::health::check_health,
            commands::config::save_api_key,
            commands::config::load_config,
            commands::install_flow::start_install,
            commands::install_flow::start_uninstall,
            commands::install_flow::open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
