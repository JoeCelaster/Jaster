#!/usr/bin/env bash
set -e

TMP_DIR=$(mktemp -d)

echo "📦 Downloading Jaster..."

curl -L \
  https://github.com/JoeCelaster/Jaster/releases/latest/download/jaster-linux-x86_64.tar.gz \
  -o "$TMP_DIR/jaster.tar.gz"

tar -xzf "$TMP_DIR/jaster.tar.gz" -C "$TMP_DIR"

cd "$TMP_DIR/jaster"

bash install.sh