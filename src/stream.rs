//! Streaming the tracker's window over the network, for OBS Studio (and anything that plays a motion-JPEG stream).
//!
//! While the "Stream the tracker to the network" option is on, a small web server listens on every network
//! interface at STREAM_PORT and serves:
//! - `/stream.mjpg` — the window as a live motion-JPEG stream (multipart/x-mixed-replace). In OBS: add a Media Source,
//!   untick "Local File", Input = `http://<this computer>:8765/stream.mjpg`, Input Format = `mpjpeg`.
//! - `/` — a page showing that stream, for an OBS Browser Source (width and height = the tracker's window size).
//! - `/frame.jpg` — one picture of the window.
//! Frames are only captured while someone is watching, at about STREAM_FPS a second. Nothing can be changed through
//! it: it only sends pictures.

use eframe::egui::ColorImage;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
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
    frames: SyncSender<Arc<ColorImage>>,
    port: u16,
}

impl Streamer {
    /// Starts listening on every interface at `port`. Fails if the port is taken.
    pub fn start(port: u16) -> std::io::Result<Streamer> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        listener.set_nonblocking(true)?;
        let shared = Arc::new(Shared { latest: Mutex::new(Latest::default()), changed: Condvar::new(), stop: AtomicBool::new(false), watchers: AtomicUsize::new(0) });

        // encoder: turns captured frames into JPEG off the drawing thread (a frame still being encoded makes the
        // next one be skipped rather than queued)
        let (tx, rx) = sync_channel::<Arc<ColorImage>>(1);
        let enc = shared.clone();
        std::thread::spawn(move || {
            while let Ok(img) = rx.recv() {
                if let Some(jpeg) = encode_jpeg(&img) {
                    let mut l = enc.latest.lock().unwrap();
                    l.seq += 1;
                    l.jpeg = Arc::new(jpeg);
                    enc.changed.notify_all();
                }
            }
        });

        let acc = shared.clone();
        std::thread::spawn(move || {
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
        Ok(Streamer { shared, frames: tx, port })
    }

    /// Whether anyone is waiting for pictures (frames are only worth capturing then).
    pub fn wanted(&self) -> bool {
        self.shared.watchers.load(Ordering::SeqCst) > 0
    }

    /// Something to hand captured frames to from another thread: a frame that arrives while the previous one is
    /// still being encoded is dropped rather than queued.
    pub fn frame_taker(&self) -> impl Fn(Arc<ColorImage>) + Send + 'static {
        let tx = self.frames.clone();
        move |img| match tx.try_send(img) {
            Ok(()) | Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {}
        }
    }

    /// The addresses to give OBS: (live stream for a Media Source, page for a Browser Source).
    pub fn urls(&self) -> (String, String) {
        urls(self.port)
    }
}

/// The addresses OBS would use on `port`: (live stream for a Media Source, page for a Browser Source).
pub fn urls(port: u16) -> (String, String) {
    let host = local_ip().unwrap_or_else(|| "localhost".into());
    (format!("http://{host}:{port}/stream.mjpg"), format!("http://{host}:{port}/"))
}

impl Drop for Streamer {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.changed.notify_all();
    }
}

/// This computer's address on the local network: the one a connection to a public address would come from. A UDP
/// "connect" only picks the route; nothing is sent.
fn local_ip() -> Option<String> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:80").ok()?;
    let ip = s.local_addr().ok()?.ip();
    (!ip.is_unspecified()).then(|| ip.to_string())
}

fn encode_jpeg(img: &ColorImage) -> Option<Vec<u8>> {
    let rgb: Vec<u8> = img.pixels.iter().flat_map(|c| { let [r, g, b, _] = c.to_srgba_unmultiplied(); [r, g, b] }).collect();
    let mut out = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    enc.encode(&rgb, img.size[0] as u32, img.size[1] as u32, image::ExtendedColorType::Rgb8).ok()?;
    Some(out)
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
    fn serves_the_page_a_frame_and_the_stream() {
        // any free port
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let st = Streamer::start(port).unwrap();
        assert!(!st.wanted());
        // stands in for the tracker: a frame whenever someone is waiting
        let take = st.frame_taker();
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
    }
}
