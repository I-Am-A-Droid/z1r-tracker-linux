//! Build step: embeds the overworld map screens (assets/overworld/questQ_xXX_yY.png) in the program, so it needs no
//! picture files when it runs. Writes a table of (quest, x, y, bytes) for src/art.rs.

use std::fmt::Write as _;

fn main() {
    let dir = std::path::Path::new("assets/overworld");
    println!("cargo:rerun-if-changed=assets/overworld");
    let mut tiles = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            // questQ_xXX_yY.png
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
    let mut code = String::from("/// Overworld map screens embedded at build time: (quest, x, y, PNG bytes).\npub const OVERWORLD_TILES: &[(usize, usize, usize, &[u8])] = &[\n");
    for (q, x, y, name) in &tiles {
        let _ = writeln!(code, "    ({q}, {x}, {y}, include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/overworld/{name}\"))),");
    }
    code.push_str("];\n");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("overworld_tiles.rs");
    std::fs::write(out, code).unwrap();
}
