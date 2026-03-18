use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use super::env_detect::EnvInfo;

const NPM_MIRROR: &str = "https://registry.npmmirror.com";
const NPM_OFFICIAL: &str = "https://registry.npmjs.org";

/// Probe which npm registry is faster. Prefer China mirror if reachable.
fn pick_registry() -> &'static str {
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

/// After npm install, scan bin dir and node_modules to find the actual openclaw binary.
/// The package might register a bin with a different name.
fn find_openclaw_binary(env: &EnvInfo) -> Option<PathBuf> {
    let bin_dir = if cfg!(target_os = "windows") {
        env.npm_prefix()
    } else {
        env.npm_prefix().join("bin")
    };

    // First: check the expected name
    let expected = env.openclaw_bin();
    if expected.exists() {
        return Some(expected);
    }

    // Second: scan bin dir for anything openclaw-related
    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("openclaw") || name.contains("open-claw") {
                return Some(entry.path());
            }
        }
    }

    // Third: check if the package installed but bin has a different name
    // Look in node_modules/.package-lock.json or the package's package.json
    let pkg_dir = env.npm_prefix().join("lib").join("node_modules");
    if let Ok(entries) = fs::read_dir(&pkg_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("openclaw") || name.contains("open-claw") {
                // Found the package, read its package.json to find bin name
                let pkg_json = entry.path().join("package.json");
                if let Ok(content) = fs::read_to_string(&pkg_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(bin) = json.get("bin") {
                            // bin can be a string or an object
                            match bin {
                                serde_json::Value::String(_) => {
                                    // bin points to a file, binary name = package name
                                    let pkg_name = json.get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("openclaw");
                                    let bin_path = bin_dir.join(pkg_name);
                                    if bin_path.exists() {
                                        return Some(bin_path);
                                    }
                                }
                                serde_json::Value::Object(map) => {
                                    for (bin_name, _) in map {
                                        let bin_path = bin_dir.join(bin_name);
                                        if bin_path.exists() {
                                            return Some(bin_path);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// List contents of bin directory for debugging.
fn list_bin_dir(env: &EnvInfo) -> String {
    let bin_dir = if cfg!(target_os = "windows") {
        env.npm_prefix()
    } else {
        env.npm_prefix().join("bin")
    };

    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.flatten() {
            items.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    if items.is_empty() {
        format!("(bin dir {} is empty or missing)", bin_dir.display())
    } else {
        format!("bin dir contents: {}", items.join(", "))
    }
}

/// Run `npm install -g openclaw@latest` using the bundled Node.js.
pub fn install_openclaw(env: &EnvInfo) -> Result<PathBuf, String> {
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    log::info!("npm stdout: {}", stdout);
    log::info!("npm stderr: {}", stderr);

    if !output.status.success() {
        return Err(format!("npm install failed:\n{}\n{}", stdout, stderr));
    }

    // Find the actual binary
    match find_openclaw_binary(env) {
        Some(bin_path) => {
            log::info!("openclaw binary found at: {}", bin_path.display());
            Ok(bin_path)
        }
        None => {
            let listing = list_bin_dir(env);
            Err(format!(
                "npm install succeeded but openclaw binary not found.\n{}\nnpm output: {}",
                listing, stdout
            ))
        }
    }
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
