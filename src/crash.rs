//! The crash log and crash recovery, as the original's (Program.fs; checked in the shipped Z1R_WPF.exe):
//! - a crash appends to Z1R_Tracker_crash_log.txt "\n\nBEGIN CRASH LOG -- <program name> -- <time>\n" (once), each
//!   line of the report, then "END CRASH LOG\n<time>\n" plus "(during startup)\n" if the tracker never finished
//!   starting (Program.logCrashInfo@238, token 0x0600000a, RVA 0x25f8; finishCrashInfoImpl@246, token 0x0600000b,
//!   RVA 0x26b4), times as yyyy-MM-dd-HH:mm:ss;
//! - at the next start, if the log's last line is a crash time less than 10 minutes ago and the autosave was written
//!   after 2 minutes before it, the tracker offers to load the autosave (Program/MyWindow::.ctor, token 0x0600004d,
//!   RVA 0x50f0: `TimeSpan.FromMinutes(10.)`, `FromMinutes(2.)`). Answering ends the log with a line that isn't a
//!   time, so it isn't offered twice.
//! The log is kept beside the saves (the original keeps both in its own folder).

use chrono::{Local, NaiveDateTime};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

pub const CRASH_LOG_NAME: &str = "Z1R_Tracker_crash_log.txt";
/// The original's dateTimeFormat "yyyy-MM-dd-HH:mm:ss".
const TIME_FORMAT: &str = "%Y-%m-%d-%H:%M:%S";
/// A crash this recent (minutes) offers recovery; an autosave written this long (minutes) before it still counts.
pub const RECENT_CRASH_MINUTES: i64 = 10;
pub const AUTOSAVE_BEFORE_CRASH_MINUTES: i64 = 2;
/// The recovery question and its answers (the original's DoModalMessageBox text and buttons).
pub const RECOVERY_TEXT: &str = "It appears that Z-Tracker recently crashed.\n\nThere is a recent auto-save from a\nminute or so before the crash.\n\nWould you like to try loading the auto-save?";
pub const RECOVERY_YES: &str = "Yes, load auto-save";
pub const RECOVERY_NO: &str = "No";
pub const RELOADED: &str = "successful reload of autosave";
pub const DECLINED: &str = "prompted for crash recovery, user chose not to, successfully started";

static LOGGED_ANY: AtomicBool = AtomicBool::new(false);
static GOT_THRU_STARTUP: AtomicBool = AtomicBool::new(false);

pub fn log_path() -> PathBuf {
    crate::platform::save_dir().join(CRASH_LOG_NAME)
}

fn now_text() -> String {
    Local::now().format(TIME_FORMAT).to_string()
}

fn append(text: &str) {
    let _ = std::fs::create_dir_all(crate::platform::save_dir());
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = f.write_all(text.as_bytes());
    }
}

/// One line of a crash report (the first one starts the report).
pub fn log_crash_info(s: &str) {
    eprintln!("{s}");
    if !LOGGED_ANY.swap(true, Ordering::SeqCst) {
        append(&format!("\n\nBEGIN CRASH LOG -- {} -- {}\n", crate::data::PROGRAM_NAME, now_text()));
    }
    append(&format!("{s}\n"));
}

/// Ends the log: "END CRASH LOG", the time, then `extra` ("(during startup)\n" instead if it never finished starting).
pub fn finish_crash_info(extra: &str) {
    let final_text = if GOT_THRU_STARTUP.load(Ordering::SeqCst) { extra } else { "(during startup)\n" };
    append(&format!("END CRASH LOG\n{}\n{final_text}", now_text()));
}

/// The tracker has finished starting (its window is drawing).
pub fn got_thru_startup() {
    GOT_THRU_STARTUP.store(true, Ordering::SeqCst);
}

/// Writes a crash report when the program fails, as the original's unhandled-exception handlers do, then lets the
/// usual report happen too.
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let main = std::thread::current().name() == Some("main");
        log_crash_info(if main { "An unhandled exception from UI thread:" } else { "An unhandled exception from background thread:" });
        log_crash_info(&info.to_string());
        finish_crash_info("");
        if GOT_THRU_STARTUP.load(Ordering::SeqCst) {
            eprintln!("\nIf Z-Tracker crashed while you were playing, you might be able to restore the most recent\nauto-save, by restarting Z-Tracker right now.");
        }
        previous(info);
    }));
}

/// Whether to offer the autosave: the log's last line is a crash time within RECENT_CRASH_MINUTES of `now`, and the
/// autosave was written after AUTOSAVE_BEFORE_CRASH_MINUTES before that crash.
pub fn offer_recovery(log: &str, now: NaiveDateTime, autosave_written: Option<NaiveDateTime>) -> bool {
    let lines: Vec<&str> = log.lines().collect();
    if lines.len() <= 1 {
        return false;
    }
    let Ok(crash) = NaiveDateTime::parse_from_str(lines[lines.len() - 1], TIME_FORMAT) else { return false };
    if now - crash >= chrono::Duration::minutes(RECENT_CRASH_MINUTES) {
        return false;
    }
    autosave_written.is_some_and(|t| t > crash - chrono::Duration::minutes(AUTOSAVE_BEFORE_CRASH_MINUTES))
}

/// offer_recovery for this computer's log and autosave.
pub fn should_offer_recovery(autosave: &std::path::Path) -> bool {
    let Ok(log) = std::fs::read_to_string(log_path()) else { return false };
    let written = std::fs::metadata(autosave).and_then(|m| m.modified()).ok().map(|t: SystemTime| chrono::DateTime::<Local>::from(t).naive_local());
    offer_recovery(&log, Local::now().naive_local(), written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, TIME_FORMAT).unwrap()
    }

    #[test]
    fn recovery_is_offered_only_after_a_recent_crash_with_a_recent_autosave() {
        let log = "\n\nBEGIN CRASH LOG -- Z-Tracker v2.1.0 -- 2026-11-08-10:00:00\nAn unhandled exception from UI thread:\nboom\nEND CRASH LOG\n2026-11-08-10:00:05\n";
        let now = t("2026-11-08-10:05:00");
        assert!(offer_recovery(log, now, Some(t("2026-11-08-09:59:00"))));
        // the autosave is too old (more than 2 minutes before the crash), or missing
        assert!(!offer_recovery(log, now, Some(t("2026-11-08-09:58:00"))));
        assert!(!offer_recovery(log, now, None));
        // the crash is 10 minutes old or more
        assert!(!offer_recovery(log, t("2026-11-08-10:10:05"), Some(t("2026-11-08-10:00:00"))));
        // a crash during startup, or one already answered, ends with a line that isn't a time
        assert!(!offer_recovery(&format!("{log}(during startup)\n"), now, Some(t("2026-11-08-10:00:00"))));
        assert!(!offer_recovery(&format!("END CRASH LOG\n2026-11-08-10:01:00\n{RELOADED}"), now, Some(t("2026-11-08-10:00:00"))));
        assert!(!offer_recovery("2026-11-08-10:00:05", now, Some(t("2026-11-08-10:00:00"))));
    }
}
