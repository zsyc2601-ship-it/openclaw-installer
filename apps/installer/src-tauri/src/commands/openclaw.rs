use std::process::Command;
use std::time::Duration;

use super::env_detect::EnvInfo;

const NPM_MIRROR: &str = "https://registry.npmmirror.com";
const NPM_OFFICIAL: &str = "https://registry.npmjs.org";

/// Probe which npm registry is faster. Prefer China mirror if reachable.
fn pick_registry() -> &'static str {
    // Try China mirror first with 3s timeout
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return NPM_OFFICIAL,
    };

    match client.head(NPM_MIRROR).send() {
        Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
            log::info!("Using China npm mirror: {}", NPM_MIRROR);
            NPM_MIRROR
        }
        _ => {
            log::info!("Using official npm registry: {}", NPM_OFFICIAL);
            NPM_OFFICIAL
        }
    }
}

/// Run `npm install -g openclaw@latest` using the bundled Node.js.
pub fn install_openclaw(env: &EnvInfo) -> Result<(), String> {
    let npm = env.npm_bin();
    let prefix = env.npm_prefix();

    if !npm.exists() {
        return Err(format!("npm not found at: {}", npm.display()));
    }

    let registry = pick_registry();
    log::info!(
        "Installing openclaw via npm (registry={}) at prefix {}",
        registry,
        prefix.display()
    );

    let output = Command::new(npm.as_os_str())
        .args([
            "install",
            "-g",
            "openclaw@latest",
            "--prefix",
            &prefix.to_string_lossy(),
            "--registry",
            registry,
        ])
        .env("PATH", node_path_env(env))
        .output()
        .map_err(|e| format!("Failed to run npm: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("npm install failed: {}", stderr));
    }

    if !env.openclaw_bin().exists() {
        return Err(format!(
            "openclaw binary not found after install at: {}",
            env.openclaw_bin().display()
        ));
    }

    log::info!("openclaw installed successfully");
    Ok(())
}

/// Run `npm uninstall -g openclaw` using the bundled Node.js.
pub fn uninstall_openclaw(env: &EnvInfo) -> Result<(), String> {
    let npm = env.npm_bin();
    let prefix = env.npm_prefix();

    if !npm.exists() {
        log::warn!("npm not found, skipping uninstall");
        return Ok(());
    }

    log::info!("Uninstalling openclaw...");

    let output = Command::new(npm.as_os_str())
        .args([
            "uninstall",
            "-g",
            "openclaw",
            "--prefix",
            &prefix.to_string_lossy(),
        ])
        .env("PATH", node_path_env(env))
        .output()
        .map_err(|e| format!("Failed to run npm uninstall: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("npm uninstall warning: {}", stderr);
    }

    Ok(())
}

/// Construct a PATH that includes our bundled node/bin.
fn node_path_env(env: &EnvInfo) -> String {
    let node_bin_dir = if cfg!(target_os = "windows") {
        env.node_dir().to_string_lossy().to_string()
    } else {
        env.node_dir().join("bin").to_string_lossy().to_string()
    };

    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let system_path = std::env::var("PATH").unwrap_or_default();
    format!("{}{}{}", node_bin_dir, sep, system_path)
}
