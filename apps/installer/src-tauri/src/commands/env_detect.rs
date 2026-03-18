use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct EnvInfo {
    pub os: String,
    pub arch: String,
    pub app_data_dir: PathBuf,
    pub home_dir: PathBuf,
    pub disk_free_mb: u64,
}

impl EnvInfo {
    pub fn node_dir(&self) -> PathBuf {
        self.app_data_dir.join("node")
    }

    pub fn node_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.node_dir().join("node.exe")
        } else {
            self.node_dir().join("bin").join("node")
        }
    }

    pub fn npm_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.node_dir().join("npm.cmd")
        } else {
            self.node_dir().join("bin").join("npm")
        }
    }

    pub fn npm_prefix(&self) -> PathBuf {
        self.app_data_dir.join("node")
    }

    pub fn openclaw_bin(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.npm_prefix().join("openclaw.cmd")
        } else {
            self.npm_prefix().join("bin").join("openclaw")
        }
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.app_data_dir.join("logs")
    }

    pub fn openclaw_config_dir(&self) -> PathBuf {
        self.home_dir.join(".openclaw")
    }
}

pub fn detect() -> Result<EnvInfo, String> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let app_data_dir = dirs::data_dir()
        .ok_or("Cannot determine app data directory")?
        .join("OpenClawDeploy");

    let home_dir = dirs::home_dir().ok_or("Cannot determine home directory")?;

    // Simple disk space check — fallback to 0 if unavailable
    let disk_free_mb = 1024; // placeholder, actual check is platform-specific

    Ok(EnvInfo {
        os,
        arch,
        app_data_dir,
        home_dir,
        disk_free_mb,
    })
}

#[tauri::command]
pub fn detect_env() -> Result<EnvInfo, String> {
    detect()
}
