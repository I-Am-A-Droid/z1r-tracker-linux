// no console window on Windows
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! Z-Tracker for Linux: a rebuild of the Zelda 1 Randomizer tracker (Z1R_WPF) as a native Linux app.

mod art;
mod assets;
mod data;
mod input;
mod model;
mod platform;
mod settings;
mod stream;
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
  --screenshot <file.png>           draw the window, save a picture of it and exit
  --write-icon <file.png>           save the program's logo (128 x 128) and exit, e.g. for an AppImage
";

#[derive(Default)]
struct Options {
    screenshot: Option<PathBuf>,
    load: Option<PathBuf>,
    quest: Option<i64>,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut next = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--screenshot" => o.screenshot = Some(PathBuf::from(next("--screenshot")?)),
            "--write-icon" => {
                let path = next("--write-icon")?;
                let icon = art::window_icon();
                match image::save_buffer(&path, &icon.rgba, icon.width, icon.height, image::ColorType::Rgba8) {
                    Ok(()) => std::process::exit(0),
                    Err(e) => return Err(format!("could not write {path}: {e}")),
                }
            }
            "--load" => o.load = Some(PathBuf::from(next("--load")?)),
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

    let assets = Assets::new();
    let settings = Settings::load();
    let hotkeys = HotKeys::load();
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

    let scale = if view.settings.get("SmallerAppWindow") { view.settings.get_f64("SmallerAppWindowScaleFactor", 0.666).clamp(MIN_WINDOW_FACTOR, MAX_WINDOW_FACTOR) as f32 } else { 1.0 };
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(WINDOW_TITLE)
        .with_app_id("z1r-tracker")
        .with_inner_size(start_size(scale))
        .with_min_inner_size([W * 0.4, H * 0.4]);
    // the tracker's own logo in the title bar, taskbar and Dock
    viewport = viewport.with_icon(std::sync::Arc::new(art::window_icon()));
    // put the window back where it was last time, like the original (MainWindowLT)
    let placed_at = if opts.screenshot.is_none() { parse_window_lt(&view.settings.get_str("MainWindowLT")) } else { None };
    if let Some((left, top)) = placed_at {
        viewport = viewport.with_position([left as f32, top as f32]);
    }
    let native = eframe::NativeOptions {
        viewport,
        renderer: renderer(),
        ..Default::default()
    };
    let screenshot = opts.screenshot.clone();
    let result = eframe::run_native(
        "z1r-tracker",
        native,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            let mut app = TrackerApp::new(view, screenshot);
            app.placed_at = placed_at.map(|(x, y)| egui::pos2(x as f32, y as f32));
            Ok(Box::new(app))
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

/// The saved window position "left,top" (setting MainWindowLT). The original only uses it when it matches
/// ^(-?\d+),(-?\d+)$ and reads both numbers with the invariant culture (Program/MyWindow::.ctor, token 0x0600004d).
fn parse_window_lt(s: &str) -> Option<(f64, f64)> {
    let re = regex::Regex::new(r"^(-?\d+),(-?\d+)$").ok()?;
    let c = re.captures(s)?;
    Some((c[1].parse().ok()?, c[2].parse().ok()?))
}

/// What the original saves each time the window moves: "%d,%d" of the whole-number left and top, but nothing while
/// either is below -15999 (a minimised window sits at -32000) (Program/-ctor@354-61::Invoke, token 0x060000b5).
fn format_window_lt(left: f64, top: f64) -> Option<String> {
    if left < WINDOW_LT_MIN || top < WINDOW_LT_MIN {
        return None;
    }
    Some(format!("{},{}", left as i32, top as i32))
}
const WINDOW_LT_MIN: f64 = -15999.0;

/// The original's window title (Program/MyWindow::.ctor, token 0x0600004d).
const WINDOW_TITLE: &str = "Z-Tracker for Zelda 1 Randomizer";
/// The original offers window sizes 4/3, default, 5/6 and 2/3 (SmallerAppWindowScaleFactor; text in Program::menu@873).
const MIN_WINDOW_FACTOR: f64 = 2.0 / 3.0 - 0.01;
const MAX_WINDOW_FACTOR: f64 = 4.0 / 3.0 + 0.01;
/// How dark a modal window makes the tracker behind it: the original's DoModal (token 0x06002751, RVA 0x8bbac) asks
/// for black "sunglasses" at opacity 0.5, which CanvasManager.CreatePopup (token 0x0600278b, RVA 0x8c834) lays over
/// the whole window under a transparent layer that takes every click.
const MODAL_SUNGLASSES_OPACITY: f32 = 0.5;

/// How the window is drawn: on Windows with Direct3D/Vulkan through wgpu (works without good OpenGL drivers, e.g.
/// in virtual machines or remote desktops), elsewhere with OpenGL. Z1R_RENDERER=glow or =wgpu overrides it.
fn renderer() -> eframe::Renderer {
    match std::env::var("Z1R_RENDERER").as_deref() {
        Ok("glow") => eframe::Renderer::Glow,
        Ok("wgpu") => eframe::Renderer::Wgpu,
        _ if cfg!(windows) => eframe::Renderer::Wgpu,
        _ => eframe::Renderer::Glow,
    }
}

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
    show_custom_map: bool,
    custom_map_text: String,
    zoom_set: bool,
    frames_seen: u64,
    /// where the window was asked to start, and how far the system actually put it from there (some X11 window
    /// managers add the title bar), so saving takes that back off and the window doesn't creep each start
    placed_at: Option<egui::Pos2>,
    place_shift: Option<Vec2>,
    /// the web server for OBS, while the "Stream the tracker to OBS" option is on
    streamer: Option<stream::Streamer>,
    last_stream_frame: Instant,
    /// the Stream button's window, the network adapters it offers, and why streaming last failed to start
    show_stream: bool,
    stream_adapters: Vec<stream::Adapter>,
    stream_error: Option<String>,
}

impl TrackerApp {
    fn new(view: View, screenshot: Option<PathBuf>) -> Self {
        TrackerApp {
            view, last_tick: Instant::now(), last_autosave_minute: -1, last_autosave: Instant::now(), screenshot, frames: 0,
            shot_state: Arc::new(AtomicU8::new(0)), show_hotkeys: false, show_custom_map: false, custom_map_text: String::new(), zoom_set: false, frames_seen: 0,
            placed_at: None, place_shift: None, streamer: None, last_stream_frame: Instant::now(), show_stream: false, stream_adapters: vec![], stream_error: None,
        }
    }

    /// Saves the window's position whenever it moves, like the original's LocationChanged handler. Some systems
    /// (Wayland) never tell programs where their window is; then nothing is saved.
    fn remember_window_position(&mut self, ctx: &egui::Context) {
        if self.screenshot.is_some() || self.frames_seen < 3 {
            return;
        }
        let zoom = ctx.zoom_factor();
        let Some(outer) = ctx.input(|i| i.viewport().outer_rect) else { return };
        let mut pos = outer.min * zoom; // in unzoomed points, as the window was placed
        let shift = *self.place_shift.get_or_insert_with(|| self.placed_at.map(|p| pos - p).unwrap_or(Vec2::ZERO));
        pos -= shift;
        if let Some(lt) = format_window_lt(pos.x as f64, pos.y as f64) {
            if lt != self.view.settings.get_str("MainWindowLT") {
                self.view.settings.set_str("MainWindowLT", &lt);
            }
        }
    }

    fn tick(&mut self) {
        // a point in the spots-remaining history whenever the count changes (TrackerModel::allUIEventingLogic)
        if self.view.m.started {
            self.view.m.note_spots_remaining();
        }
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
            if s % 60 == 0 && s / 60 != self.last_autosave_minute {
                self.last_autosave_minute = s / 60;
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
                let files = Self::save_files();
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
            "OpenSaves" => open_path(&save_dir()),
            "StartOver" => {
                // a manual save first, so the discarded work can be loaded again, then the program's starting state:
                // a fresh tracker waiting for the overworld choice (settings and hot keys are kept)
                let name = format!("zt-save-manual-{}.json", Self::stamp());
                if self.view.m.started {
                    self.save_to(&save_dir().join(&name), false);
                }
                self.view.m = Tracker::new();
                self.view.paused = false;
                self.view.popup = None;
                self.view.restart_reminders();
                self.last_autosave_minute = -1;
                let note = if save_dir().join(&name).exists() { format!("Started over. Your previous work was saved as {name}.") } else { "Started over.".to_string() };
                self.view.show_toast(note, 6.0);
            }
            "ResetKeepMaps" => {
                // as the original: a hard save first, then the reset (OverworldItemGridUI.fs resetTrackerButton)
                let name = format!("zt-save-manual-{}.json", Self::stamp());
                let p = save_dir().join(&name);
                self.save_to(&p, false);
                if p.exists() {
                    self.view.m.reset_keep_maps();
                    self.view.restart_reminders();
                    self.view.show_toast(format!("Z-Tracker data saved to file {name}. The tracker has been reset."), 6.0);
                } else {
                    self.view.show_toast("Z-Tracker was unable to save the tracker state to a file, so nothing was reset.", 6.0);
                }
            }
            "RecorderSettingsAtLegend" => self.view.open_recorder_settings(egui::Pos2::new(500.0, view::LEGEND_Y)),
            "HotKeys" => self.show_hotkeys = true,
            "ChooseCustomMap" => {
                self.custom_map_text = self.view.m.alt_map_file.clone();
                self.show_custom_map = true;
            }
            "ShowRun" => show_run_custom(&mut self.view),
            "Stream" => {
                self.stream_adapters = stream::adapters();
                self.show_stream = true;
            }
            "Zoom" => self.zoom_set = false,
            _ => {}
        }
        ctx.request_repaint();
    }

    /// Starts or stops the OBS stream to follow its option, and while someone watches, captures the window about
    /// STREAM_FPS times a second (see stream.rs).
    fn update_stream(&mut self, ctx: &egui::Context) {
        let want = self.view.settings.get("StreamToOBS");
        let ip = stream::chosen_ip(&self.view.settings);
        // a new address (Local / Network, another adapter) restarts the server there
        if self.streamer.as_ref().is_some_and(|s| !want || s.ip() != ip) {
            self.streamer = None;
            if !want {
                self.view.show_toast("Stopped streaming to OBS.", 3.0);
            }
        }
        if want && self.streamer.is_none() {
            match stream::Streamer::start(ip, stream::STREAM_PORT) {
                Ok(s) => {
                    let (live, _page) = s.urls();
                    self.view.show_toast(format!("Streaming for OBS: {live}  (Media Source, Input Format mpjpeg)"), 20.0);
                    self.streamer = Some(s);
                    self.stream_error = None;
                }
                Err(e) => {
                    self.view.settings.set("StreamToOBS", false);
                    let why = format!("Could not stream at {ip} port {}: {e}", stream::STREAM_PORT);
                    self.view.show_toast(why.clone(), 8.0);
                    self.stream_error = Some(why);
                }
            }
        }
        let Some(s) = &self.streamer else { return };
        if s.wanted() {
            let every = std::time::Duration::from_millis(1000 / stream::STREAM_FPS);
            if self.last_stream_frame.elapsed() >= every {
                self.last_stream_frame = Instant::now();
                // the mouse's last position, drawn as the original's fake pointer (in pixels, as the picture is)
                let ppp = ctx.pixels_per_point();
                let pointer = ctx.input(|i| i.pointer.latest_pos()).map(|p| [p.x * ppp, p.y * ppp]);
                let shrink = stream::broadcast_scale(stream::broadcast_size(&self.view.settings));
                ctx.request_screenshot(s.frame_taker(pointer, ppp, shrink));
            }
            ctx.request_repaint_after(every);
        }
    }

    /// The Stream button's window: streaming on or off, Local or Network (and which adapter), the address to paste
    /// into OBS, and how to set OBS up.
    fn stream_window(&mut self, ctx: &egui::Context) {
        if !self.show_stream {
            return;
        }
        // a modal window: the tracker behind it is dimmed and takes no clicks or keys until it is closed (Close,
        // Esc, or a click outside it)
        let mut close = false;
        let sunglasses = Color32::from_black_alpha((MODAL_SUNGLASSES_OPACITY * 255.0).round() as u8);
        let shown = egui::Modal::new(egui::Id::unique("stream_to_obs")).backdrop_color(sunglasses).show(ctx, |ui| {
            ui.set_width(470.0);
            ui.heading("Stream to OBS Studio");
            ui.separator();
            let settings = &mut self.view.settings;
            let mut on = settings.get("StreamToOBS");
            if ui.checkbox(&mut on, "Streaming on").changed() {
                settings.set("StreamToOBS", on);
            }
            ui.add_space(4.0);
            ui.label("Stream to:");
            let mut network = settings.get("StreamOnNetwork");
            let was = network;
            ui.radio_value(&mut network, false, "Local: OBS is on this computer (127.0.0.1)");
            ui.radio_value(&mut network, true, "Network: OBS is on another computer");
            if network != was {
                settings.set("StreamOnNetwork", network);
                if network {
                    self.stream_adapters = stream::adapters();
                }
            }
            if network {
                ui.horizontal(|ui| {
                    ui.label("Network adapter:");
                    let current = stream::chosen_ip(settings);
                    let shown = self.stream_adapters.iter().find(|a| a.ip == current).map(|a| a.describe()).unwrap_or_else(|| current.to_string());
                    egui::ComboBox::from_id_salt("stream_adapter").selected_text(shown).width(260.0).show_ui(ui, |ui| {
                        for a in &self.stream_adapters {
                            if ui.selectable_label(a.ip == current, a.describe()).clicked() {
                                settings.set_str("StreamAddress", &a.ip.to_string());
                            }
                        }
                    });
                    if ui.button("Refresh").on_hover_text("Look for network adapters again").clicked() {
                        self.stream_adapters = stream::adapters();
                    }
                });
                if self.stream_adapters.is_empty() {
                    ui.colored_label(Color32::YELLOW, "No network adapter with an address was found; check that this computer is on a network.");
                }
            }
            // the original's three broadcast sizes (its Options: Full / 2/3 / 1/3 size broadcast)
            ui.horizontal(|ui| {
                ui.label("Size:");
                let mut size = stream::broadcast_size(settings);
                for (n, label, _, _) in stream::BROADCAST_SIZES {
                    if ui.radio_value(&mut size, n, label).changed() {
                        settings.set_i64("BroadcastWindowSize", n);
                    }
                }
            });
            ui.add_space(6.0);
            let (live, page) = stream::urls(stream::chosen_ip(settings), stream::STREAM_PORT);
            ui.strong("Address for the OBS Media Source:");
            ui.horizontal(|ui| {
                let mut text = live.clone();
                ui.add(egui::TextEdit::singleline(&mut text).desired_width(340.0).font(egui::TextStyle::Monospace));
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(live.clone());
                    self.view.show_toast("Address copied.", 2.0);
                }
            });
            match (&self.streamer, &self.stream_error) {
                (Some(s), _) => {
                    let n = s.watchers();
                    ui.colored_label(Color32::LIGHT_GREEN, if n == 0 { "Streaming. Nothing is watching yet.".to_string() } else { format!("Streaming to {n} viewer(s).") });
                }
                (None, Some(e)) => {
                    ui.colored_label(Color32::YELLOW, e);
                }
                (None, None) => {
                    ui.colored_label(Color32::GRAY, "Not streaming: tick \"Streaming on\" above.");
                }
            }
            ui.separator();
            ui.strong("Setting up OBS Studio");
            ui.label("1. Tick \"Streaming on\" above.");
            ui.label("2. In OBS, under Sources, click + and choose Media Source. Give it a name and click OK.");
            ui.label("3. Untick \"Local File\".");
            ui.label("4. Paste the address above into \"Input\".");
            ui.label("5. Set \"Input Format\" to mpjpeg, then click OK.");
            ui.label("6. Drag and resize the source in OBS as you like. It shows the whole tracker window, mouse pointer included.");
            ui.add_space(4.0);
            ui.label(format!("Or use a Browser Source with the address {page} (width and height = the tracker window's size)."));
            if network {
                ui.label(format!("OBS must be on the same network. If it can't connect, allow port {} (TCP) through this computer's firewall.", stream::STREAM_PORT));
            }
            ui.label("Close this window before going live: the stream shows what the tracker window shows.");
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                if ui.button("  Close  ").clicked() {
                    close = true;
                }
            });
        });
        if close || shown.should_close() {
            self.show_stream = false;
        }
    }

    fn windows(&mut self, ctx: &egui::Context) {
        self.stream_window(ctx);
        let mut open = self.show_custom_map;
        let mut close = false;
        egui::Window::new("Overworld picture for the blank map").open(&mut open).default_width(520.0).show(ctx, |ui| {
            ui.label("With the blank map, the tracker can show a picture of the whole overworld that you made (a .png).");
            ui.label("It is stretched to 768 x 264, and each 48 x 33 part is one screen. Leave empty for no picture.");
            ui.label("Full path of the picture:");
            ui.text_edit_singleline(&mut self.custom_map_text);
            ui.horizontal(|ui| {
                if ui.button("Use this picture").clicked() {
                    let path = self.custom_map_text.trim().to_string();
                    match self.view.assets.use_custom_map(&path) {
                        Ok(()) => {
                            self.view.m.alt_map_file = path.clone();
                            if self.view.m.quest != 4 && !path.is_empty() {
                                self.view.show_toast("Picture set; it shows when the overworld is Blank (Options).", 5.0);
                            }
                            close = true;
                        }
                        Err(e) => self.view.show_toast(format!("Could not read that picture: {e}"), 6.0),
                    }
                }
                if ui.button("No picture").clicked() {
                    let _ = self.view.assets.use_custom_map("");
                    self.view.m.alt_map_file.clear();
                    close = true;
                }
            });
        });
        self.show_custom_map = open && !close;

        let mut open = self.show_hotkeys;
        egui::Window::new("HotKeys").open(&mut open).default_size([480.0, 520.0]).vscroll(true).show(ctx, |ui| {
            ui.label(format!("Hot keys are read from {}", settings::hotkeys_path().display()));
            ui.label("They work while the mouse is over the matching thing. Edit the file, then press Reload.");
            ui.horizontal(|ui| {
                if ui.button("Reload").clicked() {
                    self.view.hotkeys = HotKeys::load();
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
        self.remember_window_position(&ctx);

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
        // --screenshot only: Z1R_SCREENSHOT_HOVER=x,y pretends the mouse is over that map screen (to picture hovering)
        if self.screenshot.is_some() {
            if let Some((x, y)) = std::env::var("Z1R_SCREENSHOT_HOVER").ok().and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some((x.trim().parse::<usize>().ok()?.min(15), y.trim().parse::<usize>().ok()?.min(7)))
            }) {
                self.view.hover = view::Target::MapTile(x, y);
            }
            // Z1R_SCREENSHOT_HOVER_ITEM=PlayerHasBombs (or …WoodArrow, …BlueCandle, …BlueRing) points at that top item
            if let Ok(k) = std::env::var("Z1R_SCREENSHOT_HOVER_ITEM") {
                let keys = ["PlayerHasWoodArrow", "PlayerHasBombs", "PlayerHasBlueCandle", "PlayerHasBlueRing", "PlayerHasBoomBook", "PlayerHasWoodSword", "PlayerHasMagicalSword"];
                if let Some(key) = keys.into_iter().find(|s| *s == k) {
                    self.view.hover = view::Target::Progress(key);
                } else if k == "WhiteSword" {
                    self.view.hover = view::Target::Box(model::BoxId::WhiteSword);
                } else if k == "SpotSummary" {
                    self.view.hover = view::Target::Button("SpotSummary");
                }
            }
            // Z1R_SCREENSHOT_POPUP_HOVER=n points at choice n of the open pop-up
            if let Some(n) = std::env::var("Z1R_SCREENSHOT_POPUP_HOVER").ok().and_then(|v| v.parse::<usize>().ok()) {
                if let Some(pop) = &mut self.view.popup {
                    pop.hover = Some(n);
                }
            }
            // Z1R_SCREENSHOT_OPEN=extras opens the Starting Items and Extra Drops panel
            if self.frames == 5 {
                match std::env::var("Z1R_SCREENSHOT_OPEN").as_deref() {
                    Ok("extras") => self.view.open_starting_items(egui::Pos2::new(view::W / 2.0, 60.0)),
                    Ok("recorder") => self.view.commands.push("RecorderSettingsAtLegend"),
                    Ok("mark") => self.view.open_mark_picker(4, 3, egui::Pos2::new(220.0, 250.0)),
                    Ok("quest-only") => self.view.ask_quest_only_dont_care(true),
                    Ok("reset") => self.view.open_reset_menu(),
                    Ok("stream") => self.view.commands.push("Stream"),
                    Ok("monsters") => {
                        // a few rooms with monsters, and the monster picker open
                        for (i, m) in [1usize, 3, 9, 14, 18, 25, 27].into_iter().enumerate() {
                            self.view.m.maps[0].rooms[i][7].monster = m;
                        }
                        self.view.open_monster(3, 4, egui::Pos2::new(200.0, 560.0));
                    }
                    _ => {}
                }
            }
        }
        self.view.paint(ui, origin);

        // the notes box
        let nr = self.view.notes_rect().translate(origin);
        let popup_open = self.view.popup.is_some();
        // while a blocking window (the Stream window) is open, the tracker takes no clicks, wheel or keys
        let blocked = self.show_stream;
        let notes = ui.put(
            nr,
            egui::TextEdit::multiline(&mut self.view.m.notes)
                .font(FontId::proportional(13.0))
                .background_color(Color32::from_rgb(16, 16, 22))
                .interactive(!popup_open && !blocked)
                .desired_width(NOTES_RECT.2),
        );
        let notes_focused = notes.has_focus();

        if blocked {
            self.view.hover = view::Target::None;
        } else {
            self.view.handle_input(&ctx, notes_focused);
        }
        if !notes_focused && !blocked {
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
        if self.screenshot.is_none() {
            self.update_stream(&ctx);
        }

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
    if !path.exists() {
        let _ = std::fs::write(
            &path,
            "# Z-Tracker for Linux - ShowRunCustom file\n\
             # Lines: SHOW \"/path/to/picture.png\"  or  RUN \"/path/to/program\" arguments  or  RUN \"https://a.web/page\"\n\
             # Remove the '#' in front of a line to use it, then click Show/Run in the tracker.\n\n\
             # RUN \"https://z1r.fandom.com/wiki/Maps\"\n",
        );
    }
    let resolve = |s: &str| -> PathBuf {
        let s = s.replace('\\', "/");
        let p = PathBuf::from(&s);
        if p.is_absolute() {
            p
        } else {
            settings::config_dir().join(s.trim_start_matches("./"))
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

    #[test]
    fn window_position_rules() {
        assert_eq!(super::parse_window_lt("0,2"), Some((0.0, 2.0)));
        assert_eq!(super::parse_window_lt("-1920,40"), Some((-1920.0, 40.0)));
        assert_eq!(super::parse_window_lt(""), None);
        assert_eq!(super::parse_window_lt("10, 20"), None);
        assert_eq!(super::format_window_lt(100.7, -3.9).as_deref(), Some("100,-3"));
        assert_eq!(super::format_window_lt(-32000.0, -32000.0), None);
        assert_eq!(super::format_window_lt(-15999.0, 0.0).as_deref(), Some("-15999,0"));
    }
}
