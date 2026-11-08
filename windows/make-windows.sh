#!/bin/bash
# Makes the Windows files from Linux, in dist/ and (the same files) in windows/:
#   z1r-tracker.exe                   the program on its own
#   z1r-tracker-setup.exe             the installer (Inno Setup, run under Wine)
#   z1r-tracker-windows-x86_64.zip    the program and README in a folder
# Run from anywhere: windows/make-windows.sh
# Needs cargo-zigbuild, Wine, and Inno Setup's ISCC.exe (set ISCC=/path/to/ISCC.exe; default /tmp/z1r-inno/ISCC.exe).
# SKIP_BUILD=1 uses a program already built.
set -euo pipefail
cd "$(dirname "$0")/.."

ISCC=${ISCC:-/tmp/z1r-inno/ISCC.exe}
EXE=target/x86_64-pc-windows-gnu/release/z1r-tracker.exe
[ -n "${SKIP_BUILD:-}" ] || cargo zigbuild --release --target x86_64-pc-windows-gnu

mkdir -p dist
(cd windows && WINEDEBUG=-all wine "$ISCC" /Q installer.iss)
cp "$EXE" dist/z1r-tracker.exe

# the zip: z1r-tracker-windows-x86_64/{z1r-tracker.exe, README.md}
python3 -I - "$EXE" README.md dist/z1r-tracker-windows-x86_64.zip <<'EOF'
import sys, zipfile
exe, readme, out = sys.argv[1:4]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    z.write(exe, "z1r-tracker-windows-x86_64/z1r-tracker.exe")
    z.write(readme, "z1r-tracker-windows-x86_64/README.md")
EOF

# keep the copies beside this script current too, so whichever one is opened is the new version
cp dist/z1r-tracker.exe dist/z1r-tracker-setup.exe dist/z1r-tracker-windows-x86_64.zip windows/
echo "Made dist/z1r-tracker.exe, dist/z1r-tracker-setup.exe and dist/z1r-tracker-windows-x86_64.zip (and copies in windows/)"
