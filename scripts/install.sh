#!/usr/bin/env bash
set -e

sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/share/jaster

sudo cp jaster /usr/local/bin/jaster
sudo chmod +x /usr/local/bin/jaster

sudo cp -r assets/sounds /usr/share/jaster/

echo "✅ Jaster installed!"