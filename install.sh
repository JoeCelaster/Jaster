#!/usr/bin/env bash

set -e

echo "Installing Jaster..."

sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/share/jaster

sudo cp jaster-linux-x86_64 /usr/local/bin/jaster
sudo chmod +x /usr/local/bin/jaster

sudo cp -r assets/sounds /usr/share/jaster/

echo "✓ Installed!"