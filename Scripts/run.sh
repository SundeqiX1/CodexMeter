#!/bin/zsh
set -euo pipefail

ROOT_DIR="${0:A:h:h}"
APP_DIR="$ROOT_DIR/dist/CodexQuotaWidget.app"

if [[ ! -d "$APP_DIR" ]]; then
    "$ROOT_DIR/Scripts/build-app.sh"
fi

open "$APP_DIR"
