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

// ------------------------------------------------------------------------------------------------ Windows resources
// The original Z1R_WPF.exe carries an icon group 32512 (16, 24, 32, 48 and 64 pixel pictures), version information
// (FileVersion, ProductVersion, OriginalFilename…) and a manifest asking for no administrator rights (asInvoker). The
// Windows build gets the same: written here straight into a .res file, which both Microsoft's linker and Zig's
// (lld) take as an input, so no resource compiler is needed on either.

/// One resource in a .res file: ordinal type and name, US English, then the data padded to 4 bytes.
fn res_entry(out: &mut Vec<u8>, kind: u16, name: u16, flags: u16, data: &[u8]) {
    let mut h = vec![];
    h.extend((data.len() as u32).to_le_bytes()); // DataSize
    h.extend(32u32.to_le_bytes()); // HeaderSize
    h.extend([0xFF, 0xFF]);
    h.extend(kind.to_le_bytes());
    h.extend([0xFF, 0xFF]);
    h.extend(name.to_le_bytes());
    h.extend(0u32.to_le_bytes()); // DataVersion
    h.extend(flags.to_le_bytes()); // MemoryFlags
    h.extend(0x0409u16.to_le_bytes()); // LanguageId
    h.extend(0u32.to_le_bytes()); // Version
    h.extend(0u32.to_le_bytes()); // Characteristics
    out.extend(h);
    out.extend(data);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

fn utf16z(s: &str) -> Vec<u8> {
    s.encode_utf16().chain(std::iter::once(0)).flat_map(|c| c.to_le_bytes()).collect()
}

fn pad4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

/// A VS_VERSIONINFO-style block: wLength, wValueLength, wType, key, padding, value, padding, children.
fn ver_node(key: &str, value: &[u8], value_len: u16, text: bool, children: &[Vec<u8>]) -> Vec<u8> {
    let mut v = vec![0, 0];
    v.extend(value_len.to_le_bytes());
    v.extend((text as u16).to_le_bytes());
    v.extend(utf16z(key));
    pad4(&mut v);
    v.extend(value);
    for c in children {
        pad4(&mut v);
        v.extend(c);
    }
    let len = v.len() as u16;
    v[0..2].copy_from_slice(&len.to_le_bytes());
    v
}

/// Icon group 32512 (the original's group number) from an .ico file, version information and the asInvoker manifest.
fn windows_res(ico: &[u8], version: [u16; 4]) -> Vec<u8> {
    let mut out = vec![];
    res_entry(&mut out, 0, 0, 0, &[]); // the empty first entry every .res file starts with
    let rd16 = |o: usize| u16::from_le_bytes([ico[o], ico[o + 1]]);
    let rd32 = |o: usize| u32::from_le_bytes([ico[o], ico[o + 1], ico[o + 2], ico[o + 3]]);
    let n = if ico.len() >= 6 && rd16(2) == 1 { rd16(4) as usize } else { 0 };
    let mut group = vec![0, 0, 1, 0];
    group.extend((n as u16).to_le_bytes());
    for i in 0..n {
        let e = 6 + 16 * i;
        let (size, off) = (rd32(e + 8) as usize, rd32(e + 12) as usize);
        res_entry(&mut out, 3, (i + 1) as u16, 0x1010, &ico[off..off + size]); // RT_ICON
        group.extend(&ico[e..e + 12]); // width, height, colours, reserved, planes, bit count, size
        group.extend(((i + 1) as u16).to_le_bytes());
    }
    if n > 0 {
        res_entry(&mut out, 14, 32512, 0x1030, &group); // RT_GROUP_ICON
    }

    let [a, b, c, d] = version.map(u32::from);
    let mut fixed = vec![];
    for x in [0xFEEF04BDu32, 0x0001_0000, a << 16 | b, c << 16 | d, a << 16 | b, c << 16 | d, 0x3F, 0, 0x0004_0004, 1, 0, 0, 0] {
        fixed.extend(x.to_le_bytes());
    }
    let v = format!("{}.{}.{}.{}", version[0], version[1], version[2], version[3]);
    let strings: Vec<Vec<u8>> = [
        ("FileDescription", "Z-Tracker for Zelda 1 Randomizer"),
        ("FileVersion", v.as_str()),
        ("InternalName", "z1r-tracker"),
        ("OriginalFilename", "z1r-tracker.exe"),
        ("ProductName", "Z-Tracker"),
        ("ProductVersion", v.as_str()),
    ]
    .iter()
    .map(|(k, s)| ver_node(k, &utf16z(s), s.encode_utf16().count() as u16 + 1, true, &[]))
    .collect();
    let table = ver_node("040904b0", &[], 0, true, &strings);
    let string_info = ver_node("StringFileInfo", &[], 0, true, &[table]);
    let translation = ver_node("Translation", &0x04b0_0409u32.to_le_bytes(), 4, false, &[]);
    let var_info = ver_node("VarFileInfo", &[], 0, true, &[translation]);
    let info = ver_node("VS_VERSION_INFO", &fixed, fixed.len() as u16, false, &[string_info, var_info]);
    res_entry(&mut out, 16, 1, 0x0030, &info); // RT_VERSION

    let manifest = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#;
    res_entry(&mut out, 24, 1, 0x0030, manifest.as_bytes()); // RT_MANIFEST
    out
}

fn main() {
    // the build's date, for the About box; made again whenever the program's code or this file changes
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=Z1R_BUILD_DATE={}", build_date());
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=windows/z1r-tracker.ico");
        let ico = std::fs::read("windows/z1r-tracker.ico").unwrap_or_default();
        let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<u16>().ok()).unwrap_or(0);
        let version = [num("CARGO_PKG_VERSION_MAJOR"), num("CARGO_PKG_VERSION_MINOR"), num("CARGO_PKG_VERSION_PATCH"), 0];
        let res = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("z1r-tracker.res");
        std::fs::write(&res, windows_res(&ico, version)).unwrap();
        println!("cargo:rustc-link-arg-bins={}", res.display());
    }
    let mut code = String::new();
    table(&mut code, "Overworld map screens", "OVERWORLD_TILES", "assets/overworld");
    table(&mut code, "The magnifier's enlarged screens", "MAGNIFIER_TILES", "assets/magnifier");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("overworld_tiles.rs");
    std::fs::write(out, code).unwrap();
}
