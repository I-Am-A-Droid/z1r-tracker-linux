#!/bin/bash
# Makes dist/z1r-tracker-macos.dmg: Z-Tracker.app (runs on both Apple Silicon and Intel Macs) and a shortcut to
# Applications to drag it onto. Run on a Mac from the app folder: macos/make-dmg.sh
# Needs the Rust toolchain with both Mac targets:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
# SKIP_BUILD=1 uses programs already built in target/<target>/release.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TARGETS="aarch64-apple-darwin x86_64-apple-darwin"
APP=dist/Z-Tracker.app
DMG=dist/z1r-tracker-macos.dmg

if [ -z "${SKIP_BUILD:-}" ]; then
  for t in $TARGETS; do
    cargo build --release --target "$t"
  done
fi

rm -rf "$APP" dist/dmg "$DMG"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
# one program holding both the Apple Silicon and the Intel version
lipo -create -output "$APP/Contents/MacOS/z1r-tracker" \
  $(for t in $TARGETS; do echo "target/$t/release/z1r-tracker"; done)
sed "s/VERSION/$VERSION/g" macos/Info.plist > "$APP/Contents/Info.plist"
printf 'APPL????' > "$APP/Contents/PkgInfo"
# not signed by a registered developer: an ad-hoc signature is what Apple Silicon needs to run it at all
codesign --force --deep --sign - "$APP"

mkdir -p dist/dmg
cp -R "$APP" dist/dmg/
ln -s /Applications dist/dmg/Applications
cp README.md dist/dmg/README.txt
hdiutil create -volname "Z-Tracker" -srcfolder dist/dmg -ov -format UDZO "$DMG"
rm -rf dist/dmg
echo "Made $DMG"
