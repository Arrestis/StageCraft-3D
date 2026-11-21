#!/usr/bin/env bash
# Builds the macOS versions of Stagecraft 3D into dist/ (run on a Mac):
#   Stagecraft3D-<ver>-macos-universal.dmg   drag "Stagecraft 3D" into Applications
#   Stagecraft3D-<ver>-macos-universal.zip   the .app on its own (portable)
# One "universal" app runs natively on both Apple Silicon and Intel Macs.
#
# Needs: Rust with targets x86_64-apple-darwin and aarch64-apple-darwin
#   (rustup target add x86_64-apple-darwin aarch64-apple-darwin) and Xcode command line tools.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
DIST=dist
mkdir -p "$DIST"

cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

WORK=$(mktemp -d)
APP="$WORK/Stagecraft 3D.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/stagecraft_3d" \
    target/x86_64-apple-darwin/release/stagecraft_3d \
    target/aarch64-apple-darwin/release/stagecraft_3d
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"

# App icon (.icns) from the PNG.
ICONSET="$WORK/Stagecraft.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
    sips -z $s $s assets/icon-1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    d=$((s * 2))
    sips -z $d $d assets/icon-1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/Stagecraft.icns"

# Ad-hoc signature so it runs on Apple Silicon (not notarised: first launch needs right-click > Open).
codesign --force --deep --sign - "$APP"

ditto -c -k --keepParent "$APP" "$DIST/Stagecraft3D-$VERSION-macos-universal.zip"

DMGSRC="$WORK/dmg"
mkdir -p "$DMGSRC"
cp -R "$APP" "$DMGSRC/"
ln -s /Applications "$DMGSRC/Applications"
hdiutil create -volname "Stagecraft 3D" -srcfolder "$DMGSRC" -ov -format UDZO \
    "$DIST/Stagecraft3D-$VERSION-macos-universal.dmg"

ls -l "$DIST"
