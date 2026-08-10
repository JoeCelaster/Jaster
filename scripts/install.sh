#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/share/jaster

sudo cp "$SCRIPT_DIR/jaster" /usr/local/bin/jaster
sudo chmod +x /usr/local/bin/jaster

sudo cp -r "$SCRIPT_DIR/assets/sounds" /usr/share/jaster/

echo "✅ Jaster installed!"