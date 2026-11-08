//! Build step: embeds the overworld map screens (assets/overworld/questQ_xXX_yY.png) and the magnifier's enlarged
//! screens (assets/magnifier/questQ_xXX_yY.png) in the program, so it needs no picture files when it runs. Writes a
//! table of (quest, x, y, bytes) for each, for src/art.rs.

use std::fmt::Write as _;

/// (quest, x, y, file name) of every questQ_xXX_yY.png in `dir`, in order.
fn tiles_in(dir: &str) -> Vec<(usize, usize, usize, String)> {
    println!("cargo:rerun-if-changed={dir}");
    let mut tiles = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let Some(rest) = name.strip_prefix("quest").and_then(|r| r.strip_suffix(".png")) else { continue };
            let parts: Vec<&str> = rest.split('_').collect();
            if parts.len() != 3 {
                continue;
            }
            let num = |s: &str, p: char| s.strip_prefix(p).and_then(|v| v.parse::<usize>().ok());
            if let (Ok(q), Some(x), Some(y)) = (parts[0].parse::<usize>(), num(parts[1], 'x'), num(parts[2], 'y')) {
                tiles.push((q, x, y, name));
            }
        }
    }
    tiles.sort();
    tiles
}

fn table(code: &mut String, doc: &str, name: &str, dir: &str) {
    let _ = writeln!(code, "/// {doc}: (quest, x, y, PNG bytes), embedded at build time.");
    let _ = writeln!(code, "pub const {name}: &[(usize, usize, usize, &[u8])] = &[");
    for (q, x, y, file) in tiles_in(dir) {
        let _ = writeln!(code, "    ({q}, {x}, {y}, include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{dir}/{file}\"))),");
    }
    code.push_str("];\n");
}

/// Today's date and time (UTC) as "YYYY-MM-DD HH:MM UTC", from the system clock (days to a civil date, H. Hinnant's
/// algorithm), for the About box.
fn build_date() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02} UTC", rem / 3600, rem % 3600 / 60)
}

fn main() {
    // the build's date, for the About box; made again whenever the program's code or this file changes
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=Z1R_BUILD_DATE={}", build_date());
    let mut code = String::new();
    table(&mut code, "Overworld map screens", "OVERWORLD_TILES", "assets/overworld");
    table(&mut code, "The magnifier's enlarged screens", "MAGNIFIER_TILES", "assets/magnifier");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("overworld_tiles.rs");
    std::fs::write(out, code).unwrap();
}
