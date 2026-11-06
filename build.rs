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

fn main() {
    let mut code = String::new();
    table(&mut code, "Overworld map screens", "OVERWORLD_TILES", "assets/overworld");
    table(&mut code, "The magnifier's enlarged screens", "MAGNIFIER_TILES", "assets/magnifier");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("overworld_tiles.rs");
    std::fs::write(out, code).unwrap();
}
