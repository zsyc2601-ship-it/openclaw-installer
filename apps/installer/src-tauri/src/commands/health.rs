use std::time::Duration;

const GATEWAY_URL: &str = "http://localhost:18789";
const MAX_RETRIES: u32 = 15;
const RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// Check if the gateway is responding. Single attempt.
pub async fn ping() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(GATEWAY_URL).send().await {
        Ok(resp) => resp.status().is_success(),
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
        "Gateway did not become healthy after {} attempts",
        MAX_RETRIES
    ))
}

#[tauri::command]
pub async fn check_health() -> Result<bool, String> {
    Ok(ping().await)
}
