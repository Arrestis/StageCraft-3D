#!/usr/bin/env bash
# Builds the Linux versions of Stagecraft 3D into dist/ :
#   Stagecraft3D-<ver>-linux-x86_64.tar.gz    portable: unpack and run ./stagecraft-3d
#   Stagecraft3D-<ver>-linux-x86_64.AppImage  single file: make executable and run
#   stagecraft-3d_<ver>_amd64.deb             Debian, Ubuntu, Mint, Pop!_OS ...
#   stagecraft-3d-<ver>-1.x86_64.rpm          Fedora, openSUSE, RHEL ...
#   stagecraft-3d-<ver>-1-x86_64.pkg.tar.zst  Arch, Manjaro, EndeavourOS ...
#
# Needs (Ubuntu): libgtk-3-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev
#   libxrandr-dev libxi-dev libgl1-mesa-dev rpm libarchive-tools zstd ruby ruby-dev
#   build-essential, and `gem install fpm`. The GitHub workflow installs all of these.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
DIST=dist
mkdir -p "$DIST"
DESC="Plan and preview theatre sets in 3D inside real venue presets"
URL="https://github.com/Arrestis/StageCraft-3D"

cargo build --release
BIN=target/release/stagecraft_3d

# Portable tar.gz
WORK=$(mktemp -d)
PORT="$WORK/Stagecraft3D-$VERSION-linux-x86_64"
mkdir -p "$PORT"
install -m755 "$BIN" "$PORT/stagecraft-3d"
cp README.md packaging/linux/stagecraft-3d.desktop "$PORT/"
cp assets/icon.png "$PORT/stagecraft-3d.png"
tar -C "$WORK" -czf "$DIST/Stagecraft3D-$VERSION-linux-x86_64.tar.gz" "$(basename "$PORT")"

# File tree shared by the .deb, .rpm and Arch packages
ROOT="$WORK/root"
install -Dm755 "$BIN" "$ROOT/usr/bin/stagecraft-3d"
install -Dm644 packaging/linux/stagecraft-3d.desktop "$ROOT/usr/share/applications/stagecraft-3d.desktop"
install -Dm644 assets/icon.png "$ROOT/usr/share/icons/hicolor/512x512/apps/stagecraft-3d.png"
install -Dm644 assets/icon-256.png "$ROOT/usr/share/icons/hicolor/256x256/apps/stagecraft-3d.png"
install -Dm644 README.md "$ROOT/usr/share/doc/stagecraft-3d/README.md"

COMMON=(-s dir -n stagecraft-3d -v "$VERSION" --iteration 1 --description "$DESC" --url "$URL"
        --maintainer "Stagecraft 3D" --vendor "Stagecraft 3D" --license "All rights reserved" -C "$ROOT")
fpm "${COMMON[@]}" -t deb -a amd64 -d libgtk-3-0 -d libxkbcommon0 \
    -p "$DIST/stagecraft-3d_${VERSION}_amd64.deb" usr
fpm "${COMMON[@]}" -t rpm -a x86_64 -d gtk3 -d libxkbcommon \
    -p "$DIST/stagecraft-3d-${VERSION}-1.x86_64.rpm" usr
fpm "${COMMON[@]}" -t pacman -a x86_64 -d gtk3 -d libxkbcommon \
    -p "$DIST/stagecraft-3d-${VERSION}-1-x86_64.pkg.tar.zst" usr

# AppImage
APPDIR="$WORK/Stagecraft3D.AppDir"
install -Dm755 "$BIN" "$APPDIR/usr/bin/stagecraft-3d"
cp packaging/linux/stagecraft-3d.desktop "$APPDIR/"
cp assets/icon.png "$APPDIR/stagecraft-3d.png"
cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/stagecraft-3d" "$@"
EOF
chmod +x "$APPDIR/AppRun"
TOOL="$WORK/appimagetool"
curl -fsSL -o "$TOOL" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x "$TOOL"
ARCH=x86_64 "$TOOL" --appimage-extract-and-run "$APPDIR" "$DIST/Stagecraft3D-$VERSION-linux-x86_64.AppImage"

ls -l "$DIST"
