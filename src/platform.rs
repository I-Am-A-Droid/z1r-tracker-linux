//! The few things that differ between Linux, Windows and macOS: where settings and saves live,
//! how files and web pages are opened, and the local time for save-file names.

use std::ffi::OsStr;
use std::path::PathBuf;

const APP_DIR: &str = "z1r-tracker";

/// The user's home folder.
pub fn home() -> PathBuf {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(var).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn env_dir(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// Settings and HotKeys.txt:
/// Linux ~/.config/z1r-tracker, Windows %APPDATA%\z1r-tracker, macOS ~/Library/Application Support/z1r-tracker.
pub fn config_dir() -> PathBuf {
    let base = if cfg!(windows) {
        env_dir("APPDATA").unwrap_or_else(|| home().join("AppData").join("Roaming"))
    } else if cfg!(target_os = "macos") {
        home().join("Library").join("Application Support")
    } else {
        env_dir("XDG_CONFIG_HOME").unwrap_or_else(|| home().join(".config"))
    };
    base.join(APP_DIR)
}

/// Saves and the autosave:
/// Linux ~/.local/share/z1r-tracker/saves, Windows %LOCALAPPDATA%\z1r-tracker\saves,
/// macOS ~/Library/Application Support/z1r-tracker/saves.
pub fn save_dir() -> PathBuf {
    let base = if cfg!(windows) {
        env_dir("LOCALAPPDATA").unwrap_or_else(|| home().join("AppData").join("Local")).join(APP_DIR)
    } else if cfg!(target_os = "macos") {
        config_dir()
    } else {
        env_dir("XDG_DATA_HOME").unwrap_or_else(|| home().join(".local").join("share")).join(APP_DIR)
    };
    base.join("saves")
}

/// Opens a file, folder or web address with the system's default program.
pub fn open(target: impl AsRef<OsStr>) {
    let target = target.as_ref();
    let result = if cfg!(windows) {
        std::process::Command::new("explorer").arg(target).spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(target).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(target).spawn()
    };
    if let Err(e) = result {
        eprintln!("Could not open {}: {e}", target.to_string_lossy());
    }
}

/// Local date and time as yyyy-MM-dd-HH-mm-ss, for save-file names.
pub fn timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}
