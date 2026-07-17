#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APP_DIR="$ROOT_DIR/apps/desktop-tauri"

cd "$APP_DIR"
npm ci
npm run tauri build
