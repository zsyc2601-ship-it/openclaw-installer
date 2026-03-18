use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::env_detect::EnvInfo;

// ─── macOS: launchd ──────────────────────────────────────────

const PLIST_LABEL: &str = "com.openclaw.gateway";

fn plist_path(env: &EnvInfo) -> PathBuf {
    env.home_dir
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", PLIST_LABEL))
}

fn generate_plist(env: &EnvInfo) -> String {
    let node_bin = env.node_bin().to_string_lossy().to_string();
    let openclaw_bin = env.openclaw_bin().to_string_lossy().to_string();
    let logs_dir = env.logs_dir().to_string_lossy().to_string();
    let node_bin_dir = env.node_dir().join("bin").to_string_lossy().to_string();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{node}</string>
        <string>{openclaw}</string>
        <string>up</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{logs}/gateway.out.log</string>
    <key>StandardErrorPath</key>
    <string>{logs}/gateway.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>{node_dir}:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>"#,
        label = PLIST_LABEL,
        node = node_bin,
        openclaw = openclaw_bin,
        logs = logs_dir,
        node_dir = node_bin_dir,
    )
}

fn register_launchd(env: &EnvInfo) -> Result<(), String> {
    fs::create_dir_all(env.logs_dir())
        .map_err(|e| format!("Cannot create logs dir: {}", e))?;

    let plist = plist_path(env);
    if let Some(parent) = plist.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create LaunchAgents dir: {}", e))?;
    }

    if plist.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", &plist.to_string_lossy()])
            .output();
    }

    fs::write(&plist, generate_plist(env))
        .map_err(|e| format!("Cannot write plist: {}", e))?;

    let output = Command::new("launchctl")
        .args(["load", &plist.to_string_lossy()])
        .output()
        .map_err(|e| format!("launchctl load failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("launchctl load failed: {}", stderr));
    }

    log::info!("macOS service registered via launchd");
    Ok(())
}

fn unregister_launchd(env: &EnvInfo) -> Result<(), String> {
    let plist = plist_path(env);
    if plist.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", &plist.to_string_lossy()])
            .output();
        fs::remove_file(&plist).map_err(|e| format!("Cannot remove plist: {}", e))?;
    }
    Ok(())
}

// ─── Windows: NSSM ──────────────────────────────────────────

fn nssm_path(env: &EnvInfo) -> PathBuf {
    env.app_data_dir.join("nssm.exe")
}

fn register_nssm(env: &EnvInfo) -> Result<(), String> {
    let nssm = nssm_path(env);
    if !nssm.exists() {
        return Err(format!(
            "nssm.exe not found at {}. Bundle it in resources/nssm.exe",
            nssm.display()
        ));
    }

    fs::create_dir_all(env.logs_dir())
        .map_err(|e| format!("Cannot create logs dir: {}", e))?;

    let node = env.node_bin().to_string_lossy().to_string();
    let openclaw = env.openclaw_bin().to_string_lossy().to_string();
    let app_dir = env.app_data_dir.to_string_lossy().to_string();
    let log_out = env.logs_dir().join("gateway.out.log").to_string_lossy().to_string();
    let log_err = env.logs_dir().join("gateway.err.log").to_string_lossy().to_string();

    // Remove existing service if present (ignore errors)
    let _ = Command::new(&nssm).args(["stop", "OpenClaw"]).output();
    let _ = Command::new(&nssm).args(["remove", "OpenClaw", "confirm"]).output();

    // Install
    let args_str = format!("\"{}\" up", openclaw);
    run_cmd(&nssm, &["install", "OpenClaw", &node, &args_str])?;
    run_cmd(&nssm, &["set", "OpenClaw", "AppDirectory", &app_dir])?;
    run_cmd(&nssm, &["set", "OpenClaw", "DisplayName", "OpenClaw Gateway"])?;
    run_cmd(&nssm, &["set", "OpenClaw", "Start", "SERVICE_AUTO_START"])?;
    run_cmd(&nssm, &["set", "OpenClaw", "AppStdout", &log_out])?;
    run_cmd(&nssm, &["set", "OpenClaw", "AppStderr", &log_err])?;

    // Start
    run_cmd(&nssm, &["start", "OpenClaw"])?;

    log::info!("Windows service registered via NSSM");
    Ok(())
}

fn unregister_nssm(env: &EnvInfo) -> Result<(), String> {
    let nssm = nssm_path(env);
    if !nssm.exists() {
        log::warn!("nssm.exe not found, skipping service removal");
        return Ok(());
    }
    let _ = Command::new(&nssm).args(["stop", "OpenClaw"]).output();
    let _ = Command::new(&nssm).args(["remove", "OpenClaw", "confirm"]).output();
    log::info!("Windows service removed");
    Ok(())
}

// ─── Linux: systemd user service ─────────────────────────────

const SYSTEMD_SERVICE_NAME: &str = "openclaw-gateway.service";

fn systemd_unit_path(env: &EnvInfo) -> PathBuf {
    env.home_dir
        .join(".config")
        .join("systemd")
        .join("user")
        .join(SYSTEMD_SERVICE_NAME)
}

fn generate_systemd_unit(env: &EnvInfo) -> String {
    let node_bin = env.node_bin().to_string_lossy().to_string();
    let openclaw_bin = env.openclaw_bin().to_string_lossy().to_string();
    let node_bin_dir = env.node_dir().join("bin").to_string_lossy().to_string();

    format!(
        r#"[Unit]
Description=OpenClaw AI Gateway
After=network.target

[Service]
Type=simple
ExecStart={node} {openclaw} up
Restart=always
RestartSec=5
Environment=PATH={node_dir}:/usr/local/bin:/usr/bin:/bin

[Install]
WantedBy=default.target
"#,
        node = node_bin,
        openclaw = openclaw_bin,
        node_dir = node_bin_dir,
    )
}

fn register_systemd(env: &EnvInfo) -> Result<(), String> {
    fs::create_dir_all(env.logs_dir())
        .map_err(|e| format!("Cannot create logs dir: {}", e))?;

    let unit_path = systemd_unit_path(env);
    if let Some(parent) = unit_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create systemd user dir: {}", e))?;
    }

    fs::write(&unit_path, generate_systemd_unit(env))
        .map_err(|e| format!("Cannot write systemd unit: {}", e))?;

    run_cmd_str("systemctl", &["--user", "daemon-reload"])?;
    run_cmd_str("systemctl", &["--user", "enable", SYSTEMD_SERVICE_NAME])?;
    run_cmd_str("systemctl", &["--user", "start", SYSTEMD_SERVICE_NAME])?;

    log::info!("Linux service registered via systemd --user");
    Ok(())
}

fn unregister_systemd(env: &EnvInfo) -> Result<(), String> {
    let unit_path = systemd_unit_path(env);
    let _ = Command::new("systemctl")
        .args(["--user", "stop", SYSTEMD_SERVICE_NAME])
        .output();
    let _ = Command::new("systemctl")
        .args(["--user", "disable", SYSTEMD_SERVICE_NAME])
        .output();
    if unit_path.exists() {
        fs::remove_file(&unit_path).map_err(|e| format!("Cannot remove unit file: {}", e))?;
    }
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();
    log::info!("Linux service removed");
    Ok(())
}

// ─── Public API ──────────────────────────────────────────────

pub fn register_service(env: &EnvInfo) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        register_launchd(env)
    } else if cfg!(target_os = "windows") {
        register_nssm(env)
    } else {
        register_systemd(env)
    }
}

pub fn unregister_service(env: &EnvInfo) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        unregister_launchd(env)
    } else if cfg!(target_os = "windows") {
        unregister_nssm(env)
    } else {
        unregister_systemd(env)
    }
}

// ─── Helpers ─────────────────────────────────────────────────

fn run_cmd(program: &PathBuf, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", program.display(), e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} failed: {}", program.display(), stderr));
    }
    Ok(())
}

fn run_cmd_str(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", program, e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} failed: {}", program, stderr));
    }
    Ok(())
}
