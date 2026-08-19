#!/usr/bin/env bash
set -e

# macOS ships bash 3.2, so nothing here may use associative arrays, ${var,,}
# or mapfile.
case "$(uname -s)" in
  Linux)
    case "$(uname -m)" in
      x86_64) ASSET="jaster-linux-x86_64.tar.gz" ;;
      *)
        echo "Jaster has no Linux build for $(uname -m) yet." >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    # One universal binary covers Apple Silicon and Intel, so there is nothing
    # to decide from `uname -m` here.
    ASSET="jaster-macos-universal.tar.gz"
    ;;
  *)
    echo "Jaster supports Linux, macOS and Windows. This is $(uname -s)." >&2
    exit 1
    ;;
esac

TMP_DIR=$(mktemp -d)

echo "📦 Downloading Jaster..."

# -f so a 404 fails here, rather than writing an HTML error page into the
# tarball and failing inside `tar` with something unreadable.
curl -fL \
  "https://github.com/JoeCelaster/Jaster/releases/latest/download/$ASSET" \
  -o "$TMP_DIR/jaster.tar.gz"

tar -xzf "$TMP_DIR/jaster.tar.gz" -C "$TMP_DIR"

cd "$TMP_DIR/jaster"

bash install.sh
