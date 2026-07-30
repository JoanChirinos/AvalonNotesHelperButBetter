#!/usr/bin/env bash
set -euo pipefail

# Deploy Avalon Notes Helper: rebuild backend + frontend, restart backend.
# Frontend is served under /anh3/ (nginx root symlinks to frontend/dist), so
# the base path MUST be baked in at build time or assets 404 -> blank screen.

REPO="$HOME/AvalonNotesHelperButBetter"
NODE_BIN="$HOME/.nvm/versions/node/v24.15.0/bin"
LAUNCHD_LABEL="com.joanchirinos.anh3"

export PATH="$NODE_BIN:$PATH"
source "$HOME/.cargo/env"

echo "==> Building backend (cargo build --release)"
cd "$REPO/backend"
cargo build --release

echo "==> Building frontend (VITE_BASE_PATH=/anh3/)"
cd "$REPO/frontend"
npm install
VITE_BASE_PATH=/anh3/ npm run build

echo "==> Restarting backend (launchctl kickstart -k)"
launchctl kickstart -k "gui/$(id -u)/$LAUNCHD_LABEL"

echo "==> Done. Verifying public URL..."
sleep 1
curl -s -o /dev/null -w "index.html: %{http_code}\n" https://app.joanchirinos.com/anh3/

echo "==> Deploy complete."
