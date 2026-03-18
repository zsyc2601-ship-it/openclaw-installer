use std::time::Duration;

const GATEWAY_URL: &str = "http://localhost:18789";
const MAX_RETRIES: u32 = 30;
const RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// Check if the gateway is responding. Single attempt.
/// Must bypass system proxy to avoid false positives from v2ray/clash etc.
pub async fn ping() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .no_proxy() // Critical: bypass system proxy
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(GATEWAY_URL).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            // 200-499 = openclaw is running (503 from proxy ≠ openclaw)
            // openclaw typically returns 200 or 401/403 before API key config
            status < 500
        }
        Err(_) => false,
    }
}

/// Wait for the gateway to become healthy, with retries.
pub async fn wait_for_healthy() -> Result<(), String> {
    for attempt in 1..=MAX_RETRIES {
        log::info!("Health check attempt {}/{}...", attempt, MAX_RETRIES);
        if ping().await {
            log::info!("Gateway is healthy!");
            return Ok(());
        }
        tokio::time::sleep(RETRY_INTERVAL).await;
    }
    Err(format!(
        "Gateway did not respond at {} after {} attempts ({}s). \
         Check logs: ~/Library/Application Support/OpenClawDeploy/logs/",
        GATEWAY_URL,
        MAX_RETRIES,
        MAX_RETRIES * RETRY_INTERVAL.as_secs() as u32
    ))
}

#[tauri::command]
pub async fn check_health() -> Result<bool, String> {
    Ok(ping().await)
}
