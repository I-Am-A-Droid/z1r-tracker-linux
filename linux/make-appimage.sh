#!/bin/bash
# Makes dist/Z-Tracker-x86_64.AppImage: one file that runs on most 64-bit Linux systems (double-click or
# ./Z-Tracker-x86_64.AppImage). Run from anywhere: linux/make-appimage.sh
# Needs appimagetool (https://github.com/AppImage/appimagetool; or set APPIMAGETOOL=/path/to/appimagetool).
# With cargo-zigbuild installed the program is built against glibc 2.28, so it also runs on older distributions
# (Debian 10, Ubuntu 18.10, RHEL 8 and newer); otherwise it is built for this system's glibc.
# SKIP_BUILD=1 uses a program already built.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
APPIMAGETOOL=${APPIMAGETOOL:-appimagetool}
APPDIR=dist/Z-Tracker.AppDir
OUT=dist/Z-Tracker-x86_64.AppImage

if command -v cargo-zigbuild >/dev/null 2>&1; then
  BIN=target/x86_64-unknown-linux-gnu/release/z1r-tracker
  [ -n "${SKIP_BUILD:-}" ] || cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.28
else
  BIN=target/release/z1r-tracker
  [ -n "${SKIP_BUILD:-}" ] || cargo build --release
fi

mkdir -p dist
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
cp "$BIN" "$APPDIR/usr/bin/z1r-tracker"
# the program draws its own logo; save it as the AppImage's icon
"$APPDIR/usr/bin/z1r-tracker" --write-icon "$APPDIR/z1r-tracker.png"
cp "$APPDIR/z1r-tracker.png" "$APPDIR/.DirIcon"

cat > "$APPDIR/z1r-tracker.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Z-Tracker
GenericName=Zelda 1 Randomizer tracker
Comment=Item and map tracker for the Zelda 1 Randomizer
Exec=z1r-tracker
Icon=z1r-tracker
Categories=Game;
Terminal=false
StartupWMClass=z1r-tracker
X-AppImage-Version=$VERSION
EOF

cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/z1r-tracker" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# full paths: appimagetool may run from another folder (e.g. with APPIMAGE_EXTRACT_AND_RUN=1)
ARCH=x86_64 "$APPIMAGETOOL" --no-appstream "$PWD/$APPDIR" "$PWD/$OUT"
echo "Made $OUT"
