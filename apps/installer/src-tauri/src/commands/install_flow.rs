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
        StepProgress::new(1, "检测系统环境", "检查操作系统、架构、磁盘空间..."),
    );
    let env = env_detect::detect()?;
    log::info!("Environment: {:?}", env);

    // Ensure app data directory exists
    std::fs::create_dir_all(&env.app_data_dir)
        .map_err(|e| format!("Cannot create app data dir: {}", e))?;

    // Step 2: Extract Node.js
    send(
        &on_progress,
        StepProgress::new(2, "释放 Node.js 运行时", "解压内嵌的 Node.js 22..."),
    );
    node_setup::setup_node(&app_handle, &env)?;

    // Step 3: Install OpenClaw
    send(
        &on_progress,
        StepProgress::new(3, "安装 OpenClaw", "npm install -g openclaw@latest..."),
    );
    openclaw::install_openclaw(&env)?;

    // Step 4: Register system service
    send(
        &on_progress,
        StepProgress::new(4, "注册系统服务", "配置开机自启..."),
    );
    service::register_service(&env)?;

    // Step 5: Start and wait for gateway
    send(
        &on_progress,
        StepProgress::new(5, "启动 Gateway", "等待服务就绪..."),
    );
    health::wait_for_healthy().await?;

    // Step 6: Done — need API key
    send(
        &on_progress,
        StepProgress::new(6, "安装完成", "请配置 API Key"),
    );

    Ok("awaiting_api_key".to_string())
}

#[tauri::command]
pub async fn start_uninstall(
    remove_data: bool,
    on_progress: Channel<StepProgress>,
) -> Result<(), String> {
    let env = env_detect::detect()?;

    // Step 1: Stop service
    send(
        &on_progress,
        StepProgress::new(1, "停止服务", "停止 OpenClaw Gateway..."),
    );
    service::unregister_service(&env)?;

    // Step 2: Uninstall openclaw
    send(
        &on_progress,
        StepProgress::new(2, "卸载 OpenClaw", "npm uninstall -g openclaw..."),
    );
    openclaw::uninstall_openclaw(&env)?;

    // Step 3: Remove Node.js
    send(
        &on_progress,
        StepProgress::new(3, "清理运行时", "移除内嵌 Node.js..."),
    );
    node_setup::remove_node(&env)?;

    // Step 4: Optionally remove user data
    if remove_data {
        send(
            &on_progress,
            StepProgress::new(4, "删除用户数据", "移除 ~/.openclaw/..."),
        );
        let config_dir = env.openclaw_config_dir();
        if config_dir.exists() {
            std::fs::remove_dir_all(&config_dir)
                .map_err(|e| format!("Cannot remove config dir: {}", e))?;
        }
    } else {
        send(
            &on_progress,
            StepProgress::new(4, "保留用户数据", "配置和聊天记录已保留"),
        );
    }

    // Step 5: Clean up app data
    send(
        &on_progress,
        StepProgress::new(5, "清理完成", "所有组件已移除"),
    );

    // Remove logs
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
