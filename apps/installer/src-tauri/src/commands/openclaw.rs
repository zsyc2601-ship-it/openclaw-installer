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

/// Get the path to npm-cli.js (more reliable than running npm shell script directly).
fn npm_cli_js(env: &EnvInfo) -> PathBuf {
    env.node_dir()
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("bin")
        .join("npm-cli.js")
}

/// Build environment variables for running node/npm.
fn build_env(env: &EnvInfo) -> Vec<(String, String)> {
    let node_bin_dir = if cfg!(target_os = "windows") {
        env.node_dir().to_string_lossy().to_string()
    } else {
        env.node_dir().join("bin").to_string_lossy().to_string()
    };

    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let system_path = std::env::var("PATH").unwrap_or_default();

    vec![
        ("PATH".to_string(), format!("{}{}{}", node_bin_dir, sep, system_path)),
        ("HOME".to_string(), env.home_dir.to_string_lossy().to_string()),
        ("TMPDIR".to_string(), std::env::temp_dir().to_string_lossy().to_string()),
    ]
}

/// After npm install, scan bin dir and node_modules to find the actual openclaw binary.
fn find_openclaw_binary(env: &EnvInfo) -> Option<PathBuf> {
    let bin_dir = if cfg!(target_os = "windows") {
        env.npm_prefix()
    } else {
        env.npm_prefix().join("bin")
    };

    // Check expected name first
    let expected = env.openclaw_bin();
    if expected.exists() {
        return Some(expected);
    }

    // Scan bin dir for anything openclaw-related
    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("openclaw") || name.contains("open-claw") {
                return Some(entry.path());
            }
        }
    }

    // Check node_modules for the package and read its bin field
    let pkg_dir = env.npm_prefix().join("lib").join("node_modules");
    if let Ok(entries) = fs::read_dir(&pkg_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("openclaw") || name.contains("open-claw") {
                let pkg_json = entry.path().join("package.json");
                if let Ok(content) = fs::read_to_string(&pkg_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(bin) = json.get("bin") {
                            match bin {
                                serde_json::Value::String(_) => {
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

/// List contents of bin dir and node_modules for debugging.
fn debug_dirs(env: &EnvInfo) -> String {
    let bin_dir = if cfg!(target_os = "windows") {
        env.npm_prefix()
    } else {
        env.npm_prefix().join("bin")
    };
    let modules_dir = env.npm_prefix().join("lib").join("node_modules");

    let mut parts = Vec::new();

    // List bin dir
    let mut bin_items = Vec::new();
    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.flatten() {
            bin_items.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    parts.push(format!("bin/: [{}]", bin_items.join(", ")));

    // List node_modules
    let mut mod_items = Vec::new();
    if let Ok(entries) = fs::read_dir(&modules_dir) {
        for entry in entries.flatten() {
            mod_items.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    parts.push(format!("lib/node_modules/: [{}]", mod_items.join(", ")));

    parts.join("\n")
}

/// Run `npm install -g openclaw@latest` using the bundled Node.js.
/// Uses `node npm-cli.js` instead of running npm script directly for reliability.
pub fn install_openclaw(env: &EnvInfo) -> Result<PathBuf, String> {
    let node = env.node_bin();
    let npm_cli = npm_cli_js(env);
    let prefix = env.npm_prefix();

    if !node.exists() {
        return Err(format!("node not found at: {}", node.display()));
    }

    // Fallback: if npm-cli.js doesn't exist, try the npm script
    if !npm_cli.exists() {
        log::warn!("npm-cli.js not found at {}, trying npm bin", npm_cli.display());
        return install_openclaw_via_npm_bin(env);
    }

    let registry = pick_registry();
    log::info!(
        "Installing openclaw: node {} install -g openclaw@latest --prefix {} --registry {}",
        npm_cli.display(),
        prefix.display(),
        registry,
    );

    let envs = build_env(env);
    let output = Command::new(node.as_os_str())
        .arg(npm_cli.as_os_str())
        .args([
            "install",
            "-g",
            "openclaw@latest",
            "--prefix",
            &prefix.to_string_lossy(),
            "--registry",
            registry,
        ])
        .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .output()
        .map_err(|e| format!("Failed to run node: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    log::info!("npm stdout: {}", stdout);
    log::info!("npm stderr: {}", stderr);

    if !output.status.success() {
        return Err(format!(
            "npm install failed (exit {}):\nstdout: {}\nstderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        ));
    }

    // Find the actual binary
    match find_openclaw_binary(env) {
        Some(bin_path) => {
            log::info!("openclaw binary found at: {}", bin_path.display());
            Ok(bin_path)
        }
        None => {
            let listing = debug_dirs(env);
            Err(format!(
                "npm install succeeded but openclaw binary not found.\n{}\nstdout: {}\nstderr: {}",
                listing, stdout, stderr
            ))
        }
    }
}

/// Fallback: use npm bin directly (less reliable, for when npm-cli.js path is different).
fn install_openclaw_via_npm_bin(env: &EnvInfo) -> Result<PathBuf, String> {
    let npm = env.npm_bin();
    let prefix = env.npm_prefix();

    if !npm.exists() {
        return Err(format!("npm not found at: {}", npm.display()));
    }

    let registry = pick_registry();
    let envs = build_env(env);

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
        .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .output()
        .map_err(|e| format!("Failed to run npm: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "npm install failed (exit {}):\nstdout: {}\nstderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        ));
    }

    match find_openclaw_binary(env) {
        Some(bin_path) => Ok(bin_path),
        None => {
            let listing = debug_dirs(env);
            Err(format!(
                "npm install succeeded but openclaw binary not found.\n{}\nstdout: {}\nstderr: {}",
                listing, stdout, stderr
            ))
        }
    }
}

/// Run `npm uninstall -g openclaw` using the bundled Node.js.
pub fn uninstall_openclaw(env: &EnvInfo) -> Result<(), String> {
    let node = env.node_bin();
    let npm_cli = npm_cli_js(env);
    let prefix = env.npm_prefix();

    if !node.exists() || !npm_cli.exists() {
        log::warn!("node/npm not found, skipping uninstall");
        return Ok(());
    }

    log::info!("Uninstalling openclaw...");

    let envs = build_env(env);
    let output = Command::new(node.as_os_str())
        .arg(npm_cli.as_os_str())
        .args([
            "uninstall",
            "-g",
            "openclaw",
            "--prefix",
            &prefix.to_string_lossy(),
        ])
        .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .output()
        .map_err(|e| format!("Failed to run npm uninstall: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("npm uninstall warning: {}", stderr);
    }

    Ok(())
}
