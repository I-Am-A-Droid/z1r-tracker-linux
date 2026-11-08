//! Streaming the tracker's window over the network, for OBS Studio (and anything that plays a motion-JPEG stream).
//!
//! While streaming is on (the Stream button's window, or the option), a small web server listens at STREAM_PORT on
//! the chosen address — 127.0.0.1 for OBS on this computer ("Local"), or one network adapter's address ("Network")
//! — and serves:
//! - `/stream.mjpg` — the window as a live motion-JPEG stream (multipart/x-mixed-replace). In OBS: add a Media Source,
//!   untick "Local File", Input = `http://<this computer>:8765/stream.mjpg`, Input Format = `mpjpeg`.
//! - `/` — a page showing that stream, for an OBS Browser Source (width and height = the tracker's window size).
//! - `/frame.jpg` — one picture of the window.
//! Frames are only captured while someone is watching, at about STREAM_FPS a second. Nothing can be changed through
//! it: it only sends pictures.

use eframe::egui::ColorImage;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// The port the stream is served on.
pub const STREAM_PORT: u16 = 8765;
/// Frames a second sent while someone is watching.
pub const STREAM_FPS: u64 = 15;
/// JPEG quality (1-100).
const JPEG_QUALITY: u8 = 85;
/// The pointer drawn into the stream, as the original's broadcast window draws it (screen capture doesn't show the
/// real one): a white triangle with its tip at the mouse, corners (0,0), (12,6), (6,12) in tracker units
/// (Broadcast/addFakeMouse@168-1, token 0x06000f8a, RVA 0x3895c).
pub const FAKE_POINTER: [[f32; 2]; 3] = [[0.0, 0.0], [12.0, 6.0], [6.0, 12.0]];

/// The original's broadcast sizes (`BroadcastWindowSize`), as (setting, menu label, width in tracker units, scale):
/// "Full size broadcast" 768 wide at 1, "2/3 size broadcast" 512 at 0.666666, "1/3 size broadcast" 256 at 0.333333
/// (Broadcast/makeBroadcastWindow@45, token 0x06000f8b, RVA 0x389f0; labels from OptionsMenu.makeOptionsCanvas).
pub const BROADCAST_SIZES: [(i64, &str, f32, f32); 3] = [(3, "Full size", 768.0, 1.0), (2, "2/3 size", 512.0, 0.666666), (1, "1/3 size", 256.0, 0.333333)];

/// The stream's size setting as the original reads it: missing means 3 (full size, TrackerModelOptions..cctor,
/// token 0x060008bd), and anything outside 1..=3 is brought back into it (TrackerModelOptions.read, token
/// 0x06000759, RVA 0x148d4).
pub fn broadcast_size(settings: &crate::settings::Settings) -> i64 {
    settings.get_i64("BroadcastWindowSize", 3).clamp(1, 3)
}

/// How much the stream's picture is shrunk for a size: 1 → 0.333333, 2 → 0.666666, anything else → 1.
pub fn broadcast_scale(size: i64) -> f32 {
    match size {
        1 => 0.333333,
        2 => 0.666666,
        _ => 1.0,
    }
}

/// One captured window picture, with where the mouse was (in the picture's pixels), how many pixels make one
/// tracker unit, and how much to shrink it (broadcast_scale).
struct Frame {
    img: Arc<ColorImage>,
    pointer: Option<[f32; 2]>,
    scale: f32,
    shrink: f32,
}

#[derive(Default)]
struct Latest {
    seq: u64,
    jpeg: Arc<Vec<u8>>,
}

struct Shared {
    latest: Mutex<Latest>,
    changed: Condvar,
    stop: AtomicBool,
    /// people watching the live stream, plus requests waiting for one picture
    watchers: AtomicUsize,
}

/// A running stream server.
pub struct Streamer {
    shared: Arc<Shared>,
    frames: SyncSender<Frame>,
    ip: IpAddr,
    port: u16,
    accepting: Option<std::thread::JoinHandle<()>>,
}

impl Streamer {
    /// Starts listening on `ip` at `port`. Fails if the port is taken or the address isn't this computer's.
    pub fn start(ip: IpAddr, port: u16) -> std::io::Result<Streamer> {
        let listener = TcpListener::bind((ip, port))?;
        listener.set_nonblocking(true)?;
        let shared = Arc::new(Shared { latest: Mutex::new(Latest::default()), changed: Condvar::new(), stop: AtomicBool::new(false), watchers: AtomicUsize::new(0) });

        // encoder: turns captured frames into JPEG off the drawing thread (a frame still being encoded makes the
        // next one be skipped rather than queued)
        let (tx, rx) = sync_channel::<Frame>(1);
        let enc = shared.clone();
        std::thread::spawn(move || {
            while let Ok(f) = rx.recv() {
                if let Some(jpeg) = encode_jpeg(&f.img, f.pointer, f.scale, f.shrink) {
                    let mut l = enc.latest.lock().unwrap();
                    l.seq += 1;
                    l.jpeg = Arc::new(jpeg);
                    enc.changed.notify_all();
                }
            }
        });

        let acc = shared.clone();
        let accepting = std::thread::spawn(move || {
            while !acc.stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let s = acc.clone();
                        std::thread::spawn(move || {
                            let _ = serve(stream, &s);
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(100)),
                    Err(_) => std::thread::sleep(Duration::from_millis(100)),
                }
            }
        });
        Ok(Streamer { shared, frames: tx, ip, port, accepting: Some(accepting) })
    }

    /// The address it listens on.
    pub fn ip(&self) -> IpAddr {
        self.ip
    }

    /// How many are watching the live stream (or waiting for a picture).
    pub fn watchers(&self) -> usize {
        self.shared.watchers.load(Ordering::SeqCst)
    }

    /// Whether anyone is waiting for pictures (frames are only worth capturing then).
    pub fn wanted(&self) -> bool {
        self.shared.watchers.load(Ordering::SeqCst) > 0
    }

    /// Something to hand a captured frame to from another thread, with the mouse position in the picture's pixels
    /// (drawn as FAKE_POINTER, `scale` pixels to the unit). A frame that arrives while the previous one is still
    /// being encoded is dropped rather than queued.
    pub fn frame_taker(&self, pointer: Option<[f32; 2]>, scale: f32, shrink: f32) -> impl Fn(Arc<ColorImage>) + Send + 'static {
        let tx = self.frames.clone();
        move |img| match tx.try_send(Frame { img, pointer, scale, shrink }) {
            Ok(()) | Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {}
        }
    }

    /// The addresses to give OBS: (live stream for a Media Source, page for a Browser Source).
    pub fn urls(&self) -> (String, String) {
        urls(self.ip, self.port)
    }
}

/// The addresses OBS uses for `ip` and `port`: (live stream for a Media Source, page for a Browser Source).
pub fn urls(ip: IpAddr, port: u16) -> (String, String) {
    let host = match ip {
        IpAddr::V6(v6) => format!("[{v6}]"),
        IpAddr::V4(v4) => v4.to_string(),
    };
    (format!("http://{host}:{port}/stream.mjpg"), format!("http://{host}:{port}/"))
}

impl Drop for Streamer {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.changed.notify_all();
        // wait for the listener to close (at most ~0.1 s), so the same address can be used again at once
        if let Some(t) = self.accepting.take() {
            let _ = t.join();
        }
    }
}

/// A network adapter's address that OBS on another computer could reach.
#[derive(Clone, Debug, PartialEq)]
pub struct Adapter {
    pub name: String,
    pub ip: IpAddr,
}

impl Adapter {
    pub fn describe(&self) -> String {
        format!("{} — {}", self.ip, self.name)
    }
}

/// This computer's network adapters with an IPv4 address (not the loopback one), the one the system uses to reach
/// other networks first.
pub fn adapters() -> Vec<Adapter> {
    let mut v: Vec<Adapter> = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|i| !i.is_loopback() && i.ip().is_ipv4())
        .map(|i| Adapter { ip: i.ip(), name: i.name })
        .collect();
    v.dedup();
    if let Some(main) = route_ip() {
        v.sort_by_key(|a| a.ip != main);
    }
    v
}

/// The address for "Local": OBS on this computer.
pub const LOCAL_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// Where to stream from, by the settings: "Local" (StreamOnNetwork off, the default) is 127.0.0.1; "Network" is the
/// chosen adapter's address (StreamAddress), or the main network address when none was chosen.
pub fn chosen_ip(settings: &crate::settings::Settings) -> IpAddr {
    if !settings.get("StreamOnNetwork") {
        return LOCAL_IP;
    }
    settings.get_str("StreamAddress").trim().parse::<IpAddr>().ok().or_else(route_ip).unwrap_or(LOCAL_IP)
}

/// This computer's address on the local network: the one a connection to a public address would come from. A UDP
/// "connect" only picks the route; nothing is sent.
fn route_ip() -> Option<IpAddr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:80").ok()?;
    let ip = s.local_addr().ok()?.ip();
    (!ip.is_unspecified()).then_some(ip)
}

fn encode_jpeg(img: &ColorImage, pointer: Option<[f32; 2]>, scale: f32, shrink: f32) -> Option<Vec<u8>> {
    let mut rgb: Vec<u8> = img.pixels.iter().flat_map(|c| { let [r, g, b, _] = c.to_srgba_unmultiplied(); [r, g, b] }).collect();
    if let Some(tip) = pointer {
        draw_pointer(&mut rgb, img.size, tip, scale);
    }
    let (mut w, mut h) = (img.size[0] as u32, img.size[1] as u32);
    // the smaller broadcast sizes: the whole picture (pointer too) shrunk, as the original's ScaleTransform does
    if shrink < 1.0 {
        let full = image::RgbImage::from_raw(w, h, rgb)?;
        let (sw, sh) = (((w as f32 * shrink).round() as u32).max(1), ((h as f32 * shrink).round() as u32).max(1));
        rgb = image::imageops::resize(&full, sw, sh, image::imageops::FilterType::Triangle).into_raw();
        (w, h) = (sw, sh);
    }
    let mut out = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    enc.encode(&rgb, w, h, image::ExtendedColorType::Rgb8).ok()?;
    Some(out)
}

/// Paints FAKE_POINTER in white with its tip at `tip`, edges smoothed (4 × 4 samples a pixel).
fn draw_pointer(rgb: &mut [u8], size: [usize; 2], tip: [f32; 2], scale: f32) {
    let pts = FAKE_POINTER.map(|[x, y]| [tip[0] + x * scale, tip[1] + y * scale]);
    let edge = |a: [f32; 2], b: [f32; 2], p: [f32; 2]| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
    let inside = |p: [f32; 2]| {
        let (e0, e1, e2) = (edge(pts[0], pts[1], p), edge(pts[1], pts[2], p), edge(pts[2], pts[0], p));
        (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0)
    };
    let (x0, x1) = (tip[0].floor().max(0.0) as usize, ((tip[0] + 12.0 * scale).ceil() as usize).min(size[0]));
    let (y0, y1) = (tip[1].floor().max(0.0) as usize, ((tip[1] + 12.0 * scale).ceil() as usize).min(size[1]));
    for y in y0..y1 {
        for x in x0..x1 {
            let mut n = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    n += inside([x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0]) as u32;
                }
            }
            if n > 0 {
                let i = (y * size[0] + x) * 3;
                for c in &mut rgb[i..i + 3] {
                    *c = ((*c as u32 * (16 - n) + 255 * n) / 16) as u8;
                }
            }
        }
    }
}

const PAGE: &str = "<!doctype html><html><head><meta charset=\"utf-8\"><title>Z-Tracker</title>\
<style>html,body{margin:0;background:#000;height:100%;overflow:hidden}img{width:100%;height:100%;object-fit:contain;display:block}</style>\
</head><body><img src=\"/stream.mjpg\" alt=\"Z-Tracker\"></body></html>";

/// Answers one request: the page, one frame, or the live stream until the viewer goes away.
fn serve(stream: TcpStream, s: &Shared) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    // skip the headers
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h.trim().is_empty() {
            break;
        }
    }
    let path = line.split_whitespace().nth(1).unwrap_or("/").split('?').next().unwrap_or("/").to_string();
    let mut out = stream;
    match path.as_str() {
        "/" | "/index.html" => {
            write!(out, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n{PAGE}", PAGE.len())?;
        }
        "/frame.jpg" => {
            // a fresh picture (the tracker only captures while someone is waiting); the last one if none comes
            let now = s.latest.lock().map(|l| l.seq).unwrap_or(0);
            s.watchers.fetch_add(1, Ordering::SeqCst);
            let mut got = None;
            for _ in 0..3 {
                got = wait_for_frame(s, now);
                if got.as_ref().is_some_and(|(seq, _)| *seq > now) {
                    break;
                }
            }
            s.watchers.fetch_sub(1, Ordering::SeqCst);
            let jpeg = got.map(|(_, j)| j).unwrap_or_default();
            write!(out, "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n", jpeg.len())?;
            out.write_all(&jpeg)?;
        }
        "/stream.mjpg" => {
            s.watchers.fetch_add(1, Ordering::SeqCst);
            let r = stream_frames(&mut out, s);
            s.watchers.fetch_sub(1, Ordering::SeqCst);
            r?;
        }
        _ => {
            write!(out, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")?;
        }
    }
    out.flush()
}

fn stream_frames(out: &mut TcpStream, s: &Shared) -> std::io::Result<()> {
    out.set_write_timeout(Some(Duration::from_secs(10)))?;
    write!(out, "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n")?;
    let mut seen = 0;
    while !s.stop.load(Ordering::SeqCst) {
        let Some((seq, jpeg)) = wait_for_frame(s, seen) else { continue };
        seen = seq;
        write!(out, "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", jpeg.len())?;
        out.write_all(&jpeg)?;
        out.write_all(b"\r\n")?;
        out.flush()?;
    }
    Ok(())
}

/// The newest frame after `after` (waits up to a second; then the newest there is, so an idle tracker still sends a
/// frame now and then and players don't give up).
fn wait_for_frame(s: &Shared, after: u64) -> Option<(u64, Arc<Vec<u8>>)> {
    let l = s.latest.lock().ok()?;
    let (l, _) = s.changed.wait_timeout_while(l, Duration::from_secs(1), |l| l.seq <= after && !s.stop.load(Ordering::SeqCst)).ok()?;
    (l.seq > 0).then(|| (l.seq, l.jpeg.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn the_pointer_is_the_originals_white_triangle_with_its_tip_at_the_mouse() {
        let size = [40, 40];
        let mut rgb = vec![0u8; 40 * 40 * 3];
        draw_pointer(&mut rgb, size, [10.0, 10.0], 2.0);
        let at = |x: usize, y: usize| rgb[(y * 40 + x) * 3];
        assert_eq!(at(9, 9), 0); // nothing before the tip
        assert_eq!(at(20, 20), 255); // inside: (10,10) + (5,5)·2
        assert_eq!(at(33, 33), 0); // past the far corners (34,22) and (22,34)
        assert_eq!(at(31, 21), 255); // near the (12,6) corner
        assert_eq!(at(21, 31), 255); // near the (6,12) corner
        // a pointer partly off the picture is cut, not a crash
        draw_pointer(&mut rgb, size, [35.0, -5.0], 2.0);
    }

    #[test]
    fn serves_the_page_a_frame_and_the_stream() {
        // any free port
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let st = Streamer::start(LOCAL_IP, port).unwrap();
        assert_eq!(st.urls().0, format!("http://127.0.0.1:{port}/stream.mjpg"));
        assert!(!st.wanted());
        // stands in for the tracker: a frame whenever someone is waiting
        let take = st.frame_taker(None, 1.0, 1.0);
        let shared = st.shared.clone();
        std::thread::spawn(move || {
            while !shared.stop.load(Ordering::SeqCst) {
                if shared.watchers.load(Ordering::SeqCst) > 0 {
                    take(Arc::new(ColorImage::filled([8, 6], eframe::egui::Color32::RED)));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });

        let get = |path: &str, bytes: usize| -> Vec<u8> {
            let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
            write!(c, "GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
            c.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut buf = vec![0u8; bytes];
            let mut got = 0;
            while got < bytes {
                match c.read(&mut buf[got..]) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => got += n,
                }
            }
            buf.truncate(got);
            buf
        };
        let page = String::from_utf8_lossy(&get("/", 4096)).to_string();
        assert!(page.starts_with("HTTP/1.1 200") && page.contains("/stream.mjpg"));
        let frame = get("/frame.jpg", 65536);
        let body = frame.windows(4).position(|w| w == b"\r\n\r\n").map(|i| &frame[i + 4..]).unwrap();
        assert_eq!(&body[..2], &[0xFF, 0xD8]); // a JPEG
        let live = get("/stream.mjpg", 600);
        let text = String::from_utf8_lossy(&live);
        assert!(text.contains("multipart/x-mixed-replace; boundary=frame") && text.contains("--frame"));
        assert_eq!(get("/nothing", 64).starts_with(b"HTTP/1.1 404"), true);
        // stopping frees the address straight away
        drop(st);
        let again = Streamer::start(LOCAL_IP, port).unwrap();
        drop(again);
    }

    #[test]
    fn broadcast_sizes_shrink_the_picture_as_the_original() {
        assert_eq!((broadcast_scale(1), broadcast_scale(2), broadcast_scale(3), broadcast_scale(0)), (0.333333, 0.666666, 1.0, 1.0));
        let img = ColorImage::filled([768, 967], eframe::egui::Color32::RED);
        for (size, _, width, scale) in BROADCAST_SIZES {
            let jpeg = encode_jpeg(&img, Some([10.0, 10.0]), 1.0, broadcast_scale(size)).unwrap();
            let pic = image::load_from_memory(&jpeg).unwrap();
            assert_eq!(pic.width(), width as u32);
            assert_eq!(pic.height(), (967.0 * scale).round() as u32);
        }
    }

    #[test]
    fn addresses_for_obs() {
        let ip: IpAddr = "192.168.1.20".parse().unwrap();
        assert_eq!(urls(ip, 8765), ("http://192.168.1.20:8765/stream.mjpg".to_string(), "http://192.168.1.20:8765/".to_string()));
        assert!(adapters().iter().all(|a| !a.ip.is_loopback() && a.ip.is_ipv4()));
    }
}
