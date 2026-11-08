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

/// The shortcut the original writes beside itself every time it starts (Program/MyWindow::.ctor, token 0x0600004d,
/// RVA 0x50f0, IL_0175–IL_01fe): a ShellLink with description "Launch Z-Tracker", the program as target, its folder as
/// working directory and an icon, saved as "ZTracker.lnk" in the program's folder.
pub const SHORTCUT_NAME: &str = "ZTracker.lnk";
pub const SHORTCUT_DESCRIPTION: &str = "Launch Z-Tracker";

/// A Windows shortcut (.lnk, the MS-SHLLINK format) to `target`, starting in `dir`, with `description` and the icon
/// `icon` (picture 0). The target is given by its path (LinkInfo with a local base path), so no shell is needed to
/// make it.
pub fn shell_link(target: &str, dir: &str, description: &str, icon: &str) -> Vec<u8> {
    let u32le = |v: &mut Vec<u8>, x: u32| v.extend(x.to_le_bytes());
    let mut out = vec![];
    // ShellLinkHeader
    u32le(&mut out, 0x4C);
    out.extend([0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46]); // CLSID 00021401-…-46
    let flags = 0x02 | 0x04 | 0x10 | 0x40 | 0x80; // HasLinkInfo, HasName, HasWorkingDir, HasIconLocation, IsUnicode
    u32le(&mut out, flags);
    u32le(&mut out, 0x20); // FILE_ATTRIBUTE_ARCHIVE
    out.extend([0u8; 24]); // creation, access and write times
    u32le(&mut out, 0); // file size
    u32le(&mut out, 0); // icon index
    u32le(&mut out, 1); // SW_SHOWNORMAL
    out.extend([0u8; 2 + 2 + 4 + 4]); // hot key and reserved
    // LinkInfo: a VolumeID for a fixed drive and the target's path, as ANSI (best effort) and as UTF-16
    let ansi: Vec<u8> = target.chars().map(|c| if c.is_ascii() { c as u8 } else { b'?' }).chain([0]).collect();
    let wide: Vec<u8> = target.encode_utf16().chain([0]).flat_map(|c| c.to_le_bytes()).collect();
    let volume: Vec<u8> = [0x11u32, 3, 0, 0x10].iter().flat_map(|x| x.to_le_bytes()).chain([0]).collect();
    let header = 0x24u32;
    let volume_at = header;
    let base_at = volume_at + volume.len() as u32;
    let suffix_at = base_at + ansi.len() as u32;
    let mut wide_at = suffix_at + 1;
    wide_at += wide_at % 2; // UTF-16 text on an even offset
    let wide_suffix_at = wide_at + wide.len() as u32;
    let size = wide_suffix_at + 2;
    let mut info = vec![];
    for x in [size, header, 1, volume_at, base_at, 0, suffix_at, wide_at, wide_suffix_at] {
        u32le(&mut info, x);
    }
    info.extend(&volume);
    info.extend(&ansi);
    info.push(0); // common path suffix (ANSI): empty
    while (info.len() as u32) < wide_at {
        info.push(0);
    }
    info.extend(&wide);
    info.extend([0, 0]); // common path suffix (UTF-16): empty
    out.extend(info);
    // StringData, in the format's order: name, working directory, icon location
    for s in [description, dir, icon] {
        let units: Vec<u16> = s.encode_utf16().collect();
        out.extend((units.len() as u16).to_le_bytes());
        out.extend(units.iter().flat_map(|c| c.to_le_bytes()));
    }
    u32le(&mut out, 0); // no extra data blocks
    out
}

/// Writes ZTracker.lnk beside the program, as the original does at every start (Windows only; a folder the program
/// can't write to is left alone). The shortcut's icon is the program's own built-in icon.
pub fn write_shortcut() {
    if !cfg!(windows) {
        return;
    }
    let (Ok(exe), Some(dir)) = (std::env::current_exe(), program_dir()) else { return };
    let (exe_s, dir_s) = (exe.to_string_lossy().to_string(), dir.to_string_lossy().to_string());
    let _ = std::fs::write(dir.join(SHORTCUT_NAME), shell_link(&exe_s, &dir_s, SHORTCUT_DESCRIPTION, &exe_s));
}

#[cfg(test)]
mod tests {
    #[test]
    fn recognises_the_originals_folder() {
        assert!(!super::is_original_folder(&std::env::temp_dir().join("no-such-z1r-folder")));
    }

    #[test]
    fn the_shortcut_is_a_windows_shell_link_with_the_originals_texts() {
        let target = r"C:\Games\Z-Tracker\z1r-tracker.exe";
        let lnk = super::shell_link(target, r"C:\Games\Z-Tracker", super::SHORTCUT_DESCRIPTION, target);
        let u32at = |o: usize| u32::from_le_bytes(lnk[o..o + 4].try_into().unwrap());
        assert_eq!(u32at(0), 0x4C);
        assert_eq!(&lnk[4..8], &[0x01, 0x14, 0x02, 0x00]);
        assert_eq!(u32at(20), 0x02 | 0x04 | 0x10 | 0x40 | 0x80);
        // LinkInfo: its local base path is the target, in ANSI and UTF-16
        let info = 0x4C;
        let size = u32at(info) as usize;
        let base = info + u32at(info + 16) as usize;
        assert_eq!(&lnk[base..base + target.len()], target.as_bytes());
        let wide = info + u32at(info + 28) as usize;
        let w: Vec<u16> = lnk[wide..wide + target.len() * 2].chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        assert_eq!(String::from_utf16(&w).unwrap(), target);
        // then the three strings, counted, and the terminal block
        let mut at = info + size;
        let mut read = || {
            let n = u16::from_le_bytes([lnk[at], lnk[at + 1]]) as usize;
            let w: Vec<u16> = lnk[at + 2..at + 2 + n * 2].chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            at += 2 + n * 2;
            String::from_utf16(&w).unwrap()
        };
        assert_eq!([read(), read(), read()], ["Launch Z-Tracker", r"C:\Games\Z-Tracker", target]);
        assert_eq!(&lnk[lnk.len() - 4..], &[0, 0, 0, 0]);
        assert_eq!(super::SHORTCUT_NAME, "ZTracker.lnk");
    }

    #[test]
    fn program_folder_of_a_mac_app_is_outside_it() {
        use std::path::Path;
        assert_eq!(super::bundle_folder(Path::new("/Applications/Z-Tracker.app/Contents/MacOS")), Path::new("/Applications"));
        assert_eq!(super::bundle_folder(Path::new("/home/me/ZTracker")), Path::new("/home/me/ZTracker"));
        assert_eq!(super::bundle_folder(Path::new("/x/Contents/MacOS")), Path::new("/x/Contents/MacOS"));
    }
}
