#!/bin/sh
set -e
cargo tauri build --release
DMG=$(ls target/release/bundle/dmg/*.dmg)
if [ -n "$APPLE_CERT" ]; then
  codesign --timestamp --sign "$APPLE_CERT" "$DMG"
fi
