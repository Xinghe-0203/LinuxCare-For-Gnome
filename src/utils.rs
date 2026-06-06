#![allow(dead_code)]

use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use gtk::glib;

/// Spawn a blocking task on a background thread and run a callback on the main thread.
pub fn spawn_bg<T: Send + 'static, F: FnOnce() -> T + Send + 'static, C: FnOnce(T) + 'static>(
    work: F,
    callback: C,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = work();
        let _ = tx.send(result);
    });
    let mut callback = Some(callback);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(result) => {
                if let Some(cb) = callback.take() {
                    cb(result);
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                if let Some(cb) = callback.take() {
                    drop(cb);
                }
                glib::ControlFlow::Break
            }
        }
    });
}

pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

pub fn get_desktop_dir() -> PathBuf {
    get_home_dir().join("Desktop")
}

pub fn get_applications_dir() -> PathBuf {
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    for dir in data_dirs.split(':') {
        let app_dir = Path::new(dir).join("applications");
        if app_dir.exists() {
            return app_dir;
        }
    }

    get_home_dir().join(".local/share/applications")
}

#[derive(Debug, Default, Clone)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub comment: String,
    pub categories: String,
    pub icon: String,
    pub entry_type: String,
    pub no_display: bool,
}

pub fn parse_desktop_file(path: &Path) -> Result<DesktopEntry> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    let mut entry = DesktopEntry::default();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("Name=") {
            entry.name = line[5..].to_string();
        } else if line.starts_with("Exec=") {
            entry.exec = line[5..].to_string();
        } else if line.starts_with("Comment=") {
            entry.comment = line[8..].to_string();
        } else if line.starts_with("Categories=") {
            entry.categories = line[11..].to_string();
        } else if line.starts_with("Icon=") {
            entry.icon = line[5..].to_string();
        } else if line.starts_with("Type=") {
            entry.entry_type = line[5..].to_string();
        } else if line.starts_with("NoDisplay=") {
            entry.no_display = line[10..] == *"true";
        }
    }

    Ok(entry)
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub status: String,
}

pub fn get_flatpak_packages() -> Result<Vec<PackageInfo>> {
    let output = run_command("flatpak", &["list", "--app", "--columns=name,version"])?;
    let mut packages = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            packages.push(PackageInfo {
                name: parts[0].to_string(),
                version: parts.get(1).unwrap_or(&"").to_string(),
                description: String::new(),
                status: "installed".to_string(),
            });
        }
    }

    Ok(packages)
}

pub fn get_snap_packages() -> Result<Vec<PackageInfo>> {
    let output = run_command("snap", &["list"])?;
    let mut packages = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            packages.push(PackageInfo {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                description: String::new(),
                status: "installed".to_string(),
            });
        }
    }

    Ok(packages)
}

pub fn get_apt_packages() -> Result<Vec<PackageInfo>> {
    let output = run_command("dpkg", &["-l"])?;
    let mut packages = Vec::new();

    for line in output.lines().skip(5) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            packages.push(PackageInfo {
                name: parts[1].to_string(),
                version: parts[2].to_string(),
                description: parts[3..].join(" "),
                status: parts[0].to_string(),
            });
        }
    }

    Ok(packages)
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub device: String,
    pub size: String,
    pub used: String,
    pub available: String,
    pub use_percent: String,
    pub mount_point: String,
}

pub fn get_disk_usage() -> Result<Vec<DiskInfo>> {
    let output = run_command("df", &["-h"])?;
    let mut disks = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            disks.push(DiskInfo {
                device: parts[0].to_string(),
                size: parts[1].to_string(),
                used: parts[2].to_string(),
                available: parts[3].to_string(),
                use_percent: parts[4].to_string(),
                mount_point: parts[5].to_string(),
            });
        }
    }

    Ok(disks)
}

pub fn clean_apt_cache() -> Result<String> {
    run_command("sudo", &["apt-get", "clean"])
}

pub fn clean_snap_cache() -> Result<String> {
    let cache_dir = get_home_dir().join("snap/cache");
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    Ok("Snap cache cleaned".to_string())
}

pub fn clean_flatpak_cache() -> Result<String> {
    run_command("flatpak", &["uninstall", "--unused", "-y"])
}

pub fn clean_log_files() -> Result<String> {
    let log_dir = Path::new("/var/log");
    if log_dir.exists() {
        for entry in fs::read_dir(log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "log") {
                let _ = fs::remove_file(&path);
            }
        }
    }
    Ok("Log files cleaned".to_string())
}

pub fn clean_temp_files() -> Result<String> {
    let temp_dirs = vec![
        "/tmp".to_string(),
        get_home_dir().join(".cache/thumbnails").to_string_lossy().to_string(),
    ];

    for dir in temp_dirs {
        let path = Path::new(&dir);
        if path.exists() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    Ok("Temporary files cleaned".to_string())
}
