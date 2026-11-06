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
    if let Some(dir) = original_folder() {
        return dir;
    }
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
    if let Some(dir) = original_folder() {
        return dir;
    }
    let base = if cfg!(windows) {
        env_dir("LOCALAPPDATA").unwrap_or_else(|| home().join("AppData").join("Local")).join(APP_DIR)
    } else if cfg!(target_os = "macos") {
        config_dir()
    } else {
        env_dir("XDG_DATA_HOME").unwrap_or_else(|| home().join(".local").join("share")).join(APP_DIR)
    };
    base.join("saves")
}

/// When the tracker sits in a folder that holds a Z1R_Tracker_settings.json (for example the original Z-Tracker's
/// folder, or a copy of the tracker kept on a USB stick), it keeps settings, HotKeys.txt and saves right there, as
/// the original does: it uses AppDomain.BaseDirectory for
/// Z1R_Tracker_settings.json (TrackerModelOptions::readSettings, token 0x0600075c) and for every zt-save-*.json
/// (SaveAndLoad::SaveAll, token 0x06000013; $SaveAndLoad::.cctor for the autosave). The two then share them.
pub fn original_folder() -> Option<PathBuf> {
    program_dir().filter(|d| is_original_folder(d))
}

/// The folder the program sits in. For a macOS app (Z-Tracker.app/Contents/MacOS/z1r-tracker) that is the folder
/// holding Z-Tracker.app, which is what the person sees in Finder.
pub fn program_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(bundle_folder(exe.parent()?))
}

fn bundle_folder(dir: &std::path::Path) -> PathBuf {
    let name = |p: &std::path::Path| p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if let Some(contents) = dir.parent().filter(|_| name(dir) == "MacOS") {
        if let Some(app) = contents.parent().filter(|_| name(contents) == "Contents") {
            if name(app).ends_with(".app") {
                if let Some(outside) = app.parent() {
                    return outside.to_path_buf();
                }
            }
        }
    }
    dir.to_path_buf()
}

pub fn is_original_folder(dir: &std::path::Path) -> bool {
    dir.join("Z1R_Tracker_settings.json").is_file()
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

#[cfg(test)]
mod tests {
    #[test]
    fn recognises_the_originals_folder() {
        assert!(!super::is_original_folder(&std::env::temp_dir().join("no-such-z1r-folder")));
    }

    #[test]
    fn program_folder_of_a_mac_app_is_outside_it() {
        use std::path::Path;
        assert_eq!(super::bundle_folder(Path::new("/Applications/Z-Tracker.app/Contents/MacOS")), Path::new("/Applications"));
        assert_eq!(super::bundle_folder(Path::new("/home/me/ZTracker")), Path::new("/home/me/ZTracker"));
        assert_eq!(super::bundle_folder(Path::new("/x/Contents/MacOS")), Path::new("/x/Contents/MacOS"));
    }
}
