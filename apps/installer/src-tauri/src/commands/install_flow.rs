use serde::Serialize;
use tauri::ipc::Channel;

use super::{env_detect, health, node_setup, openclaw, service};

#[derive(Clone, Serialize)]
pub struct StepProgress {
    step: u32,
    total: u32,
    label: String,
    detail: String,
}

impl StepProgress {
    fn new(step: u32, label: &str, detail: &str) -> Self {
        Self {
            step,
            total: 6,
            label: label.to_string(),
            detail: detail.to_string(),
        }
    }
}

fn send(channel: &Channel<StepProgress>, progress: StepProgress) {
    let _ = channel.send(progress);
}

#[tauri::command]
pub async fn start_install(
    app_handle: tauri::AppHandle,
    on_progress: Channel<StepProgress>,
) -> Result<String, String> {
    // Step 1: Detect environment
    send(
        &on_progress,
        StepProgress::new(1, "Detecting environment", "Checking OS, architecture, disk space..."),
    );
    let env = env_detect::detect()?;
    log::info!("Environment: {:?}", env);

    std::fs::create_dir_all(&env.app_data_dir)
        .map_err(|e| format!("Cannot create app data dir: {}", e))?;

    // Step 2: Extract Node.js
    send(
        &on_progress,
        StepProgress::new(2, "Extracting Node.js", "Unpacking bundled Node.js 22..."),
    );
    node_setup::setup_node(&app_handle, &env)?;

    // Step 3: Install OpenClaw — returns the actual binary path
    send(
        &on_progress,
        StepProgress::new(3, "Installing OpenClaw", "npm install -g openclaw@latest..."),
    );
    let openclaw_bin = openclaw::install_openclaw(&env)?;

    // Step 4: Register system service using the discovered binary path
    send(
        &on_progress,
        StepProgress::new(4, "Registering service", "Configuring auto-start..."),
    );
    service::register_service(&env, &openclaw_bin)?;

    // Step 5: Start and wait for gateway
    send(
        &on_progress,
        StepProgress::new(5, "Starting Gateway", "Waiting for service to be ready..."),
    );
    health::wait_for_healthy().await?;

    // Step 6: Done — need API key
    send(
        &on_progress,
        StepProgress::new(6, "Complete", "Please configure your API Key"),
    );

    Ok("awaiting_api_key".to_string())
}

#[tauri::command]
pub async fn start_uninstall(
    remove_data: bool,
    on_progress: Channel<StepProgress>,
) -> Result<(), String> {
    let env = env_detect::detect()?;

    send(
        &on_progress,
        StepProgress::new(1, "Stopping service", "Stopping OpenClaw Gateway..."),
    );
    service::unregister_service(&env)?;

    send(
        &on_progress,
        StepProgress::new(2, "Uninstalling OpenClaw", "npm uninstall -g openclaw..."),
    );
    openclaw::uninstall_openclaw(&env)?;

    send(
        &on_progress,
        StepProgress::new(3, "Removing runtime", "Removing Node.js..."),
    );
    node_setup::remove_node(&env)?;

    if remove_data {
        send(
            &on_progress,
            StepProgress::new(4, "Removing data", "Removing ~/.openclaw/..."),
        );
        let config_dir = env.openclaw_config_dir();
        if config_dir.exists() {
            std::fs::remove_dir_all(&config_dir)
                .map_err(|e| format!("Cannot remove config dir: {}", e))?;
        }
    } else {
        send(
            &on_progress,
            StepProgress::new(4, "Keeping data", "Config and data preserved"),
        );
    }

    send(
        &on_progress,
        StepProgress::new(5, "Done", "All components removed"),
    );

    let logs_dir = env.logs_dir();
    if logs_dir.exists() {
        let _ = std::fs::remove_dir_all(&logs_dir);
    }

    Ok(())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Cannot open URL: {}", e))
}
