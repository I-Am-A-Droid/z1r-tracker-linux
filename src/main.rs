// no console window on Windows
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! Z-Tracker for Linux: a rebuild of the Zelda 1 Randomizer tracker (Z1R_WPF) as a native Linux app.

mod assets;
mod data;
mod input;
mod model;
mod platform;
mod settings;
mod view;

use assets::Assets;
use eframe::egui::{self, Color32, FontId, Vec2};
use model::Tracker;
use settings::{save_dir, HotKeys, Settings};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;
use view::{View, H, NOTES_RECT, W};

const USAGE: &str = "Z-Tracker for Linux 2.1.0
usage: z1r-tracker [options] [save-file.json]
  --quest 1|2|mixed1|mixed2|blank   start straight away with that overworld
  --load <file.json>                open a saved tracker (the original's save files work too)
  --original <Z1R_WPF.exe>          where to read the tracker pictures from
  --screenshot <file.png>           draw the window, save a picture of it and exit
";

#[derive(Default)]
struct Options {
    screenshot: Option<PathBuf>,
    load: Option<PathBuf>,
    original: Option<String>,
    quest: Option<i64>,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut next = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--screenshot" => o.screenshot = Some(PathBuf::from(next("--screenshot")?)),
            "--load" => o.load = Some(PathBuf::from(next("--load")?)),
            "--original" => o.original = Some(next("--original")?),
            "--quest" => {
                o.quest = Some(match next("--quest")?.to_lowercase().as_str() {
                    "1" | "first" => 0,
                    "2" | "second" => 1,
                    "mixed1" => 2,
                    "mixed2" => 3,
                    "blank" => 4,
                    q => return Err(format!("unknown quest {q}")),
                })
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            s if s.starts_with('-') => return Err(format!("unknown option {s}")),
            s => o.load = Some(PathBuf::from(s)),
        }
    }
    Ok(o)
}

fn main() {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    };

    let mut assets = Assets::default();
    let remembered = Settings::load(None).get_str("LinuxOriginalProgramPath");
    let explicit = opts.original.clone().or(if remembered.is_empty() { None } else { Some(remembered) });
    if !assets.try_load(explicit.as_deref()) {
        eprintln!("The original Z1R_WPF.exe was not found; drawing plain shapes instead of its pictures.");
    }
    let settings = Settings::load(assets.original.as_deref());
    let hotkeys = HotKeys::load(assets.original.as_deref());
    let mut view = View::new(Tracker::new(), assets, settings, hotkeys);

    if let Some(path) = &opts.load {
        match load_file(&mut view, path) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Could not load {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    } else if let Some(q) = opts.quest {
        view.m.reset(q);
        view.m.started = true;
    } else if opts.screenshot.is_some() {
        view.m.reset(0);
        view.m.started = true;
    }
    if !view.assets.loaded() {
        view.show_toast("Original Z1R_WPF.exe not found: drawing plain shapes. Options > read the pictures from it.", 8.0);
    }

    let scale = if view.settings.get("SmallerAppWindow") { view.settings.get_f64("SmallerAppWindowScaleFactor", 0.666).clamp(MIN_WINDOW_FACTOR, MAX_WINDOW_FACTOR) as f32 } else { 1.0 };
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_app_id("z1r-tracker")
            .with_inner_size(start_size(scale))
            .with_min_inner_size([W * 0.4, H * 0.4]),
        ..Default::default()
    };
    let screenshot = opts.screenshot.clone();
    let result = eframe::run_native(
        "z1r-tracker",
        native,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(TrackerApp::new(view, screenshot)))
        }),
    );
    if let Err(e) = result {
        eprintln!("The window could not be opened: {e}");
        std::process::exit(1);
    }
    if let Some(code) = EXIT_CODE.get() {
        std::process::exit(*code);
    }
}

/// Starting window size: the tracker's own size (times the "smaller window" factor), or Z1R_WINDOW_SIZE=WxH if set.
fn start_size(scale: f32) -> [f32; 2] {
    std::env::var("Z1R_WINDOW_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some([w.trim().parse().ok()?, h.trim().parse().ok()?])
        })
        .unwrap_or([W * scale, H * scale])
}

/// The original's window title (Program/MyWindow::.ctor, token 0x0600004d).
const WINDOW_TITLE: &str = "Z-Tracker for Zelda 1 Randomizer";
/// The original offers window sizes 4/3, default, 5/6 and 2/3 (SmallerAppWindowScaleFactor; text in Program::menu@873).
const MIN_WINDOW_FACTOR: f64 = 2.0 / 3.0 - 0.01;
const MAX_WINDOW_FACTOR: f64 = 4.0 / 3.0 + 0.01;

static EXIT_CODE: std::sync::OnceLock<i32> = std::sync::OnceLock::new();

/// Autosave interval in seconds (WPFUI/makeAll@1981-404: DispatcherTimer.Interval = TimeSpan.FromSeconds 60).
const AUTOSAVE_SECONDS: u64 = 60;

/// How far the tracker may shrink or grow when it is scaled to the window.
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 6.0;
/// The original thinks the screen is too short for the tracker below 1000 px of usable height (times the scale factor).
const SCREEN_TOO_SHORT: f32 = 1000.0;
/// The wide arrangement is used once it shows the tracker at least 2% bigger than the tall one (and back again).
const LAYOUT_SWITCH_MARGIN: f32 = 1.02;

fn load_file(view: &mut View, path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    view.m.load_json(&text)?;
    view.paused = view.m.prog("PlayerHasRescuedZelda");
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    match version_warning(&text) {
        Some(w) => view.show_toast(w, 10.0),
        None => view.show_toast(format!("Loaded {name}"), 3.0),
    }
    Ok(())
}

/// A save made by another Z-Tracker version (Program/startButtonBehavior@1214-3, token 0x060002b3): the same message
/// as the original; it then tries to load the file anyway.
fn version_warning(text: &str) -> Option<String> {
    let root: serde_json::Value = serde_json::from_str(&model::strip_trailing_commas(text.trim_start_matches('\u{feff}'))).ok()?;
    let saved = root.get("Version")?.as_str()?.to_string();
    if saved == data::VERSION {
        return None;
    }
    let how = if saved.starts_with("1.2") || saved.starts_with("1.3") { "Loading this file might not work" } else { "Loading this file will probably fail" };
    Some(format!(
        "You are running Z-Tracker version '{}' but the save file was created using version '{saved}'. {how}, but Z-Tracker will attempt to load it anyway.",
        data::VERSION
    ))
}

struct TrackerApp {
    view: View,
    last_tick: Instant,
    last_autosave_minute: i64,
    last_autosave: Instant,
    screenshot: Option<PathBuf>,
    frames: u32,
    shot_state: Arc<AtomicU8>, // 0 not asked, 1 asked, 2 saved, 3 failed
    show_hotkeys: bool,
    show_original: bool,
    original_text: String,
    zoom_set: bool,
    frames_seen: u64,
}

impl TrackerApp {
    fn new(view: View, screenshot: Option<PathBuf>) -> Self {
        let original_text = view.assets.original.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
        TrackerApp {
            view, last_tick: Instant::now(), last_autosave_minute: -1, last_autosave: Instant::now(), screenshot, frames: 0,
            shot_state: Arc::new(AtomicU8::new(0)), show_hotkeys: false, show_original: false, original_text, zoom_set: false, frames_seen: 0,
        }
    }

    fn tick(&mut self) {
        // autosave on its own clock, every AUTOSAVE_SECONDS of real time, even while the game timer is paused
        // (WPFUI/makeAll@1981-404 starts a 60-second DispatcherTimer that calls SaveAll(AutoSave))
        if self.last_autosave.elapsed().as_secs() >= AUTOSAVE_SECONDS {
            self.last_autosave = Instant::now();
            if self.screenshot.is_none() && self.view.m.started {
                self.autosave();
                self.view.flash_disk_icon();
            }
        }
        let elapsed = self.last_tick.elapsed().as_secs();
        if elapsed == 0 {
            return;
        }
        self.last_tick += std::time::Duration::from_secs(elapsed);
        if !self.view.m.started || self.view.paused {
            return;
        }
        for _ in 0..elapsed {
            self.view.m.seconds += 1;
            let s = self.view.m.seconds;
            // once a minute of game time the remaining-spots history grows (Program/MyWindow::Update -> updateTimeline)
            if s % 60 == 0 && s / 60 != self.last_autosave_minute {
                self.last_autosave_minute = s / 60;
                let left = self.view.m.spots_remaining();
                self.view.m.spots_over_time.push(left);
            }
        }
    }

    fn save_to(&mut self, path: &Path, tell: bool) {
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")));
        match std::fs::write(path, self.view.m.to_json()) {
            Ok(()) if tell => self.view.show_toast(format!("Saved: {}", path.file_name().unwrap_or_default().to_string_lossy()), 3.0),
            Ok(()) => {}
            Err(e) => self.view.show_toast(format!("Could not save: {e}"), 6.0),
        }
    }

    fn autosave(&mut self) {
        if self.view.m.started {
            self.save_to(&save_dir().join("zt-save-zz-autosave.json"), false);
        }
    }

    fn stamp() -> String {
        platform::timestamp()
    }

    fn save_files() -> Vec<PathBuf> {
        let mut dirs = vec![save_dir()];
        if let Ok(cwd) = std::env::current_dir() {
            dirs.push(cwd);
        }
        let mut files: Vec<(std::time::SystemTime, PathBuf)> = vec![];
        for d in dirs {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("zt-save") && name.ends_with(".json") {
                    let t = e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
                    files.push((t, e.path()));
                }
            }
        }
        files.sort_by(|a, b| b.0.cmp(&a.0));
        files.dedup_by(|a, b| a.1 == b.1);
        files.into_iter().map(|f| f.1).collect()
    }

    fn run_command(&mut self, name: &str, ctx: &egui::Context) {
        match name {
            "Save" => {
                let p = save_dir().join(format!("zt-save-manual-{}.json", Self::stamp()));
                self.save_to(&p, true);
            }
            "SaveFinished" => {
                let p = save_dir().join(format!("zt-save-completed-{}.json", Self::stamp()));
                self.save_to(&p, true);
            }
            "Load" => {
                let mut files = Self::save_files();
                if let Some(orig) = self.view.assets.original.as_ref().and_then(|p| p.parent()) {
                    if let Ok(rd) = std::fs::read_dir(orig) {
                        for e in rd.flatten() {
                            let n = e.file_name().to_string_lossy().to_string();
                            if n.starts_with("zt-save") && n.ends_with(".json") && !files.contains(&e.path()) {
                                files.push(e.path());
                            }
                        }
                    }
                }
                self.view.open_load_list(files);
            }
            "LoadAutosave" => {
                let p = save_dir().join("zt-save-zz-autosave.json");
                if p.exists() {
                    if let Err(e) = load_file(&mut self.view, &p) {
                        self.view.show_toast(format!("Could not load the autosave: {e}"), 6.0);
                    }
                } else {
                    self.view.show_toast("There is no autosave yet.", 3.0);
                }
            }
            "ChooseOriginal" => self.show_original = true,
            "OpenSaves" => open_path(&save_dir()),
            "HotKeys" => self.show_hotkeys = true,
            "ShowRun" => show_run_custom(&mut self.view),
            "Zoom" => self.zoom_set = false,
            _ => {}
        }
        ctx.request_repaint();
    }

    fn windows(&mut self, ctx: &egui::Context) {
        let mut open = self.show_hotkeys;
        egui::Window::new("HotKeys").open(&mut open).default_size([480.0, 520.0]).vscroll(true).show(ctx, |ui| {
            ui.label(format!("Hot keys are read from {}", settings::hotkeys_path().display()));
            ui.label("They work while the mouse is over the matching thing. Edit the file, then press Reload.");
            ui.horizontal(|ui| {
                if ui.button("Reload").clicked() {
                    self.view.hotkeys = HotKeys::load(self.view.assets.original.as_deref());
                }
                if ui.button("Open HotKeys.txt").clicked() {
                    open_path(&settings::hotkeys_path());
                }
            });
            ui.separator();
            let hk = &self.view.hotkeys;
            for (title, table) in [("Items (hover an item box)", &hk.item), ("Overworld (hover a map tile)", &hk.overworld), ("Blockers (hover a blocker box)", &hk.blocker), ("Dungeon rooms (hover a room)", &hk.room)] {
                ui.strong(if table.is_empty() { format!("{title}: none set") } else { title.to_string() });
                let mut rows: Vec<(&settings::KeyChord, &String)> = table.iter().collect();
                rows.sort_by(|a, b| a.1.cmp(b.1));
                for (k, v) in rows {
                    ui.monospace(format!("  {:<14} = {v}", k.describe()));
                }
            }
            for p in &hk.problems {
                ui.colored_label(Color32::YELLOW, format!("Problem: {p}"));
            }
            ui.separator();
            ui.label("Always: F5 pauses/resumes the timer, Ctrl+S saves, Esc closes a pop-up.");
        });
        self.show_hotkeys = open;

        let mut open = self.show_original;
        let mut close = false;
        egui::Window::new("Pictures from the original").open(&mut open).default_width(520.0).show(ctx, |ui| {
            ui.label("The tracker's pictures are read from your copy of the original program (Z1R_WPF.exe). Nothing in it is run.");
            ui.label("Full path of Z1R_WPF.exe:");
            ui.text_edit_singleline(&mut self.original_text);
            if ui.button("Use this file").clicked() {
                let path = self.original_text.trim().to_string();
                if self.view.assets.try_load(Some(&path)) {
                    self.view.settings.set_str("LinuxOriginalProgramPath", &path);
                    self.view.show_toast("Pictures loaded.", 3.0);
                    close = true;
                } else {
                    self.view.show_toast("That file doesn't contain the tracker's pictures.", 4.0);
                }
            }
        });
        self.show_original = open && !close;
    }
}

impl eframe::App for TrackerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.zoom_set {
            // the "Smaller app window" option sets the window to that fraction of the full size
            let s = if self.view.settings.get("SmallerAppWindow") { self.view.settings.get_f64("SmallerAppWindowScaleFactor", 0.666).clamp(MIN_WINDOW_FACTOR, MAX_WINDOW_FACTOR) as f32 } else { 1.0 };
            if self.frames_seen > 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(W * s, H * s) / ctx.zoom_factor()));
            }
            self.zoom_set = true;
        }
        // a screen too short for the tracker: the original checks WorkArea.Height < 1000 × scale and only colours a bar
        // red (Program/MyWindow::.ctor); here the window starts at a size that fits instead (90% of the screen height)
        if self.frames_seen == 2 && self.screenshot.is_none() && std::env::var("Z1R_WINDOW_SIZE").is_err() {
            let s = if self.view.settings.get("SmallerAppWindow") { self.view.settings.get_f64("SmallerAppWindowScaleFactor", 0.666) as f32 } else { 1.0 };
            let zoom = ctx.zoom_factor();
            if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                let screen_h = monitor.y * zoom; // in unzoomed points
                if screen_h < SCREEN_TOO_SHORT * s {
                    let h = screen_h * 0.9;
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(W * h / H, h) / zoom));
                }
            }
        }
        self.frames_seen += 1;
        self.tick();

        // scale everything to fit the window, in whichever arrangement shows it bigger: tall (768 x 967), or wide
        // (1536 x 490, dungeons and timeline beside the map). Zoomed to the largest size that fits, keeping its shape,
        // and centered in any space left over.
        let avail = ui.max_rect();
        let zoom = ctx.zoom_factor();
        let native = avail.size() * zoom;
        let fit_for = |s: Vec2| (native.x / s.x).min(native.y / s.y);
        let (fit_tall, fit_wide) = (fit_for(Vec2::new(W, H)), fit_for(Vec2::new(view::WIDE_W, view::WIDE_H)));
        let allow_wide = self.view.settings.get("WideLayoutWhenRoomy");
        // a little stickiness so it doesn't flip back and forth at the boundary
        self.view.wide = allow_wide && if self.view.wide { fit_wide * LAYOUT_SWITCH_MARGIN >= fit_tall } else { fit_wide > fit_tall * LAYOUT_SWITCH_MARGIN };
        let size = self.view.size();
        let fit = fit_for(size).clamp(MIN_ZOOM, MAX_ZOOM);
        if (fit - zoom).abs() > 0.002 {
            ctx.set_zoom_factor(fit);
            ctx.request_repaint();
        }
        let spare = Vec2::new((avail.width() - size.x).max(0.0), (avail.height() - size.y).max(0.0)) / 2.0;
        let origin = (avail.min + spare).to_vec2().round();
        self.view.paint(ui, origin);

        // the notes box
        let nr = self.view.notes_rect().translate(origin);
        let popup_open = self.view.popup.is_some();
        let notes = ui.put(
            nr,
            egui::TextEdit::multiline(&mut self.view.m.notes)
                .font(FontId::proportional(13.0))
                .background_color(Color32::from_rgb(16, 16, 22))
                .interactive(!popup_open)
                .desired_width(NOTES_RECT.2),
        );
        let notes_focused = notes.has_focus();

        self.view.handle_input(&ctx, notes_focused);
        if !notes_focused {
            let (f5, ctrl_s) = ctx.input(|i| (i.key_pressed(egui::Key::F5), i.modifiers.command && i.key_pressed(egui::Key::S)));
            if f5 {
                self.view.paused = !self.view.paused;
            }
            if ctrl_s {
                self.view.commands.push("Save");
            }
        }
        let cmds: Vec<&'static str> = std::mem::take(&mut self.view.commands);
        for c in cmds {
            self.run_command(c, &ctx);
        }
        if let Some(p) = self.view.load_request.take() {
            if let Err(e) = load_file(&mut self.view, &p) {
                self.view.show_toast(format!("Could not load that file: {e}"), 6.0);
            }
            self.last_autosave_minute = self.view.m.seconds / 60;
        }
        self.windows(&ctx);

        // --screenshot: draw about 120 frames, then save the window's picture and exit
        if let Some(path) = self.screenshot.clone() {
            self.frames += 1;
            match self.shot_state.load(Ordering::SeqCst) {
                0 if self.frames >= 120 => {
                    self.shot_state.store(1, Ordering::SeqCst);
                    let state = self.shot_state.clone();
                    let ctx2 = ctx.clone();
                    ctx.request_screenshot(move |img| {
                        let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| { let [r, g, b, _] = c.to_srgba_unmultiplied(); [r, g, b, 255] }).collect();
                        let ok = image::save_buffer(&path, &rgba, img.size[0] as u32, img.size[1] as u32, image::ColorType::Rgba8);
                        match ok {
                            Ok(()) => {
                                println!("Saved screenshot to {}", path.display());
                                state.store(2, Ordering::SeqCst);
                            }
                            Err(e) => {
                                eprintln!("Screenshot failed: {e}");
                                state.store(3, Ordering::SeqCst);
                            }
                        }
                        ctx2.request_repaint();
                    });
                }
                2 | 3 => {
                    if self.shot_state.load(Ordering::SeqCst) == 3 {
                        let _ = EXIT_CODE.set(1);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => {}
            }
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.screenshot.is_none() {
            self.autosave();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }
}

fn open_path(p: &Path) {
    platform::open(p);
}

/// The "Show/Run" button: reads ShowRunCustom.txt (same format as the original) and opens
/// pictures, web pages or programs.
fn show_run_custom(view: &mut View) {
    let path = settings::config_dir().join("ShowRunCustom.txt");
    let orig_dir = view.assets.original.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf());
    if !path.exists() {
        match orig_dir.as_ref().map(|d| d.join("ShowRunCustom.txt")).filter(|p| p.exists()) {
            Some(o) => {
                let _ = std::fs::copy(o, &path);
            }
            None => {
                let _ = std::fs::write(
                    &path,
                    "# Z-Tracker for Linux - ShowRunCustom file\n\
                     # Lines: SHOW \"/path/to/picture.png\"  or  RUN \"/path/to/program\" arguments  or  RUN \"https://a.web/page\"\n\
                     # Remove the '#' in front of a line to use it, then click Show/Run in the tracker.\n\n\
                     # RUN \"https://z1r.fandom.com/wiki/Maps\"\n",
                );
            }
        }
    }
    let resolve = |s: &str| -> PathBuf {
        let s = s.replace('\\', "/");
        let p = PathBuf::from(&s);
        if p.is_absolute() {
            p
        } else {
            orig_dir.clone().unwrap_or_else(settings::config_dir).join(s.trim_start_matches("./"))
        }
    };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let lines = match parse_show_run(&text) {
        Ok(l) => l,
        Err(n) => {
            // the original's message (ShowRunCustom::ParseShowRunCustomFile)
            view.show_toast(
                format!(
                    "Error parsing '{}', line {n}. You should fix this error by editing the text file. Or you can delete it, and pressing the button again will create an empty template file in its place.",
                    path.display()
                ),
                10.0,
            );
            return;
        }
    };
    let mut done = 0;
    for line in lines {
        match line {
            ShowRun::Show(file) => {
                let f = resolve(&file);
                if f.exists() {
                    open_path(&f);
                    done += 1;
                } else {
                    view.show_toast(format!("Picture not found: {}", f.display()), 5.0);
                }
            }
            ShowRun::Run(target, rest) => {
                if target.starts_with("http://") || target.starts_with("https://") {
                    platform::open(&target);
                    done += 1;
                } else {
                    match std::process::Command::new(resolve(&target)).args(rest.split_whitespace()).spawn() {
                        Ok(_) => done += 1,
                        Err(e) => view.show_toast(format!("Could not run {target}: {e}"), 5.0),
                    }
                }
            }
        }
    }
    if done == 0 {
        view.show_toast(format!("Nothing to show or run yet: edit {}", path.display()), 6.0);
    }
}

/// One action from ShowRunCustom.txt.
#[derive(Debug, PartialEq)]
enum ShowRun {
    Show(String),
    Run(String, String),
}

/// Reads ShowRunCustom.txt with the original's rules (ShowRunCustom::ParseShowRunCustomFile, token 0x06002649,
/// RVA 0x87b8c): each line is blank, a "#" comment, SHOW "file", left,top,width,height SHOW "file" (the window
/// position is the original's; here pictures open in the system's viewer), or RUN "target" arguments.
/// Anything else is an error, reported with its line number (1-based).
fn parse_show_run(text: &str) -> Result<Vec<ShowRun>, usize> {
    use regex::Regex;
    let blank = Regex::new(r"^\s*$").unwrap();
    let comment = Regex::new(r"^#(.*)$").unwrap();
    let show = Regex::new(r#"^\s*SHOW\s*"(.*)"\s*$"#).unwrap();
    let show_at = Regex::new(r#"^\s*(-?\d+),(-?\d+),(\d+),(\d+)\s*SHOW\s*"(.*)"\s*$"#).unwrap();
    let run = Regex::new(r#"^\s*RUN\s*"(.*)"\s*(.*)$"#).unwrap();
    let mut out = vec![];
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r').trim_start_matches('\u{feff}');
        if blank.is_match(line) || comment.is_match(line) {
            continue;
        }
        if let Some(c) = show.captures(line) {
            out.push(ShowRun::Show(c[1].to_string()));
        } else if let Some(c) = show_at.captures(line) {
            out.push(ShowRun::Show(c[5].to_string()));
        } else if let Some(c) = run.captures(line) {
            out.push(ShowRun::Run(c[1].to_string(), c[2].trim().to_string()));
        } else {
            return Err(i + 1);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{parse_show_run, ShowRun};

    #[test]
    fn show_run_file_follows_the_originals_rules() {
        let text = "# comment\n\n SHOW \".\\pics\\a.png\"\n-5,10,300,200 SHOW \"b.png\"\nRUN \"https://x.y\" \nRUN \"tool.exe\" -a -b\n";
        assert_eq!(
            parse_show_run(text),
            Ok(vec![
                ShowRun::Show(".\\pics\\a.png".into()),
                ShowRun::Show("b.png".into()),
                ShowRun::Run("https://x.y".into(), "".into()),
                ShowRun::Run("tool.exe".into(), "-a -b".into()),
            ])
        );
        assert_eq!(parse_show_run("# ok\nshow \"lowercase is not allowed\""), Err(2));
        assert_eq!(parse_show_run("SHOW \"a.png\" extra"), Err(1));
    }

    #[test]
    fn warns_about_saves_from_other_versions_like_the_original() {
        assert_eq!(super::version_warning(r#"{"Version": "2.1.0"}"#), None);
        let old = super::version_warning(r#"{"Version": "1.3.2",}"#).unwrap();
        assert!(old.contains("version '1.3.2'") && old.contains("Loading this file might not work"));
        let other = super::version_warning(r#"{"Version": "2.0.3"}"#).unwrap();
        assert!(other.contains("Loading this file will probably fail"));
    }
}
