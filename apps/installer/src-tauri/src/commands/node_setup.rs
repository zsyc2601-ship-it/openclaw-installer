use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use tar::Archive;
use tauri::Manager;
use xz2::read::XzDecoder;

use super::env_detect::EnvInfo;

/// Locate the bundled Node.js archive from Tauri resources.
fn find_node_archive(app_handle: &tauri::AppHandle, env: &EnvInfo) -> Result<PathBuf, String> {
    let archive_name = match (env.os.as_str(), env.arch.as_str()) {
        ("macos", "aarch64") => "node-v22.22.1-darwin-arm64.tar.xz",
        ("macos", "x86_64") => "node-v22.22.1-darwin-x64.tar.xz",
        ("linux", "x86_64") => "node-v22.22.1-linux-x64.tar.xz",
        ("linux", "aarch64") => "node-v22.22.1-linux-arm64.tar.xz",
        ("windows", "x86_64") => "node-v22.22.1-win-x64.zip",
        _ => return Err(format!("Unsupported platform: {}-{}", env.os, env.arch)),
    };

    // Tauri bundles "resources/*" into Contents/Resources/resources/ on macOS,
    // so we need the "resources/" prefix in the resolve path.
    let resource_subpath = format!("resources/{}", archive_name);
    let resource_path = app_handle
        .path()
        .resolve(&resource_subpath, tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Cannot resolve resource path: {}", e))?;

    if !resource_path.exists() {
        return Err(format!(
            "Node.js archive not found at: {}",
            resource_path.display()
        ));
    }

    Ok(resource_path)
}

/// Extract Node.js tar.xz archive to the target directory (macOS/Linux).
fn extract_tar_xz(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file =
        fs::File::open(archive_path).map_err(|e| format!("Cannot open archive: {}", e))?;

    let decoder = XzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Cannot create target directory: {}", e))?;

    // Strip the top-level directory (e.g., "node-v22.22.1-darwin-arm64/")
    let entries = archive
        .entries()
        .map_err(|e| format!("Cannot read archive entries: {}", e))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Archive entry error: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("Invalid path in archive: {}", e))?
            .into_owned();

        let components: Vec<_> = path.components().collect();
        if components.len() <= 1 {
            continue;
        }

        let relative: PathBuf = components[1..].iter().collect();
        let dest = target_dir.join(&relative);

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&dest)
                .map_err(|e| format!("Cannot create dir {}: {}", dest.display(), e))?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create parent dir: {}", e))?;
            }
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("Cannot read entry: {}", e))?;
            fs::write(&dest, &buf)
                .map_err(|e| format!("Cannot write {}: {}", dest.display(), e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mode) = entry.header().mode() {
                    let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(mode));
                }
            }
        }
    }

    Ok(())
}

/// Extract Node.js .zip archive to the target directory (Windows).
fn extract_zip(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file =
        fs::File::open(archive_path).map_err(|e| format!("Cannot open archive: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Cannot read zip archive: {}", e))?;

    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Cannot create target directory: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;

        let raw_path = entry
            .enclosed_name()
            .ok_or_else(|| "Invalid path in zip".to_string())?
            .to_owned();

        // Strip top-level directory (e.g., "node-v22.22.1-win-x64/")
        let components: Vec<_> = raw_path.components().collect();
        if components.len() <= 1 {
            continue;
        }
        let relative: PathBuf = components[1..].iter().collect();
        let dest = target_dir.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&dest)
                .map_err(|e| format!("Cannot create dir {}: {}", dest.display(), e))?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create parent dir: {}", e))?;
            }
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("Cannot read zip entry: {}", e))?;
            fs::write(&dest, &buf)
                .map_err(|e| format!("Cannot write {}: {}", dest.display(), e))?;
        }
    }

    Ok(())
}

/// Clear macOS quarantine attributes recursively.
#[cfg(target_os = "macos")]
fn clear_quarantine(dir: &Path) -> Result<(), String> {
    std::process::Command::new("xattr")
        .args(["-cr", &dir.to_string_lossy()])
        .output()
        .map_err(|e| format!("xattr failed: {}", e))?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn clear_quarantine(_dir: &Path) -> Result<(), String> {
    Ok(())
}

/// Main entry: extract bundled Node.js to app_data/node/
pub fn setup_node(app_handle: &tauri::AppHandle, env: &EnvInfo) -> Result<(), String> {
    let node_dir = env.node_dir();

    if env.node_bin().exists() {
        log::info!("Node.js already exists at {}", node_dir.display());
        return Ok(());
    }

    if node_dir.exists() {
        fs::remove_dir_all(&node_dir)
            .map_err(|e| format!("Cannot clean up node dir: {}", e))?;
    }

    let archive_path = find_node_archive(app_handle, env)?;

    log::info!(
        "Extracting Node.js from {} to {}",
        archive_path.display(),
        node_dir.display()
    );

    let path_str = archive_path.to_string_lossy();
    if path_str.ends_with(".tar.xz") {
        extract_tar_xz(&archive_path, &node_dir)?;
    } else if path_str.ends_with(".zip") {
        extract_zip(&archive_path, &node_dir)?;
    } else {
        return Err(format!("Unknown archive format: {}", path_str));
    }

    clear_quarantine(&node_dir)?;

    if !env.node_bin().exists() {
        return Err(format!(
            "Node binary not found after extraction: {}",
            env.node_bin().display()
        ));
    }

    log::info!("Node.js setup complete");
    Ok(())
}

/// Remove the extracted Node.js directory.
pub fn remove_node(env: &EnvInfo) -> Result<(), String> {
    let node_dir = env.node_dir();
    if node_dir.exists() {
        fs::remove_dir_all(&node_dir)
            .map_err(|e| format!("Cannot remove node dir: {}", e))?;
    }
    Ok(())
}
