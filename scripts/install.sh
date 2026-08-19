#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# /usr/share is on macOS's sealed system volume, where SIP refuses writes even
# to root. /usr/local is the one prefix Apple leaves alone. Has to agree with
# `installed_sounds()` in src/utils/paths.rs.
if [ "$(uname -s)" = "Darwin" ]; then
  SHARE_DIR=/usr/local/share/jaster
else
  SHARE_DIR=/usr/share/jaster
fi

sudo mkdir -p /usr/local/bin
sudo mkdir -p "$SHARE_DIR"

# Staged, then renamed into place: `jaster update` runs this script from the
# very binary we are replacing, and writing over a running executable fails
# with "Text file busy". A rename swaps the file instead of rewriting it.
sudo cp "$SCRIPT_DIR/jaster" /usr/local/bin/jaster.new
sudo chmod +x /usr/local/bin/jaster.new
sudo mv -f /usr/local/bin/jaster.new /usr/local/bin/jaster

# Removed first because `cp -r src/sounds dst/` copies *into* an existing
# sounds/ rather than over it, so every update since launch has been nesting
# another sounds/sounds/ inside the last one.
sudo rm -rf "$SHARE_DIR/sounds"
sudo cp -r "$SCRIPT_DIR/assets/sounds" "$SHARE_DIR/"

# `jaster update` sets this. It reports the new version and restarts the daemon
# itself, so the first-run welcome below would only be noise on top of that.
if [ -n "${JASTER_UPDATE:-}" ]; then
  exit 0
fi

# Colors
GREEN=$(printf '\033[32m')
CYAN=$(printf '\033[36m')
YELLOW=$(printf '\033[33m')
GRAY=$(printf '\033[90m')
BOLD=$(printf '\033[1m')
RESET=$(printf '\033[0m')

# The one part of the welcome that is not the same on both Unixes: Linux needs
# a group, macOS needs a permission granted to the terminal.
if [ "$(uname -s)" = "Darwin" ]; then
  PERMISSION="${YELLOW}Let Jaster see your keyboard${RESET}

    ${GRAY}System Settings -> Privacy & Security -> Input Monitoring${RESET}

    Turn on the entry for your terminal, then quit it completely (Cmd-Q)
    and reopen it. ${GRAY}macOS grants this to whatever launched jaster,
    not to jaster, so there may be no \"jaster\" entry in the list.${RESET}
"
else
  PERMISSION="${YELLOW}If no keyboards are detected${RESET}

    sudo usermod -aG input \$USER
    exec su - \"\$USER\"
"
fi

cat <<EOF

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

                  ${BOLD}${CYAN} Jaster is Ready! ${RESET}

${YELLOW}Get Started${RESET}

    ${GREEN}jaster start${RESET}

${YELLOW}Volume${RESET}   -   ${GRAY}starts at 150${RESET}

    ${GRAY}60 for headphones${RESET}   ${GREEN}jaster volume 60${RESET}
    ${GRAY}150 for speakers${RESET}    ${GREEN}jaster volume 150${RESET}

${YELLOW}Available Commands${RESET}

    ${YELLOW}jaster doctor${RESET}    ${GRAY}Check installation${RESET}
    ${GREEN}jaster sounds${RESET}    ${GRAY}List installed sound packs and their shortcuts${RESET}
    ${GREEN}jaster oreo${RESET}      ${GRAY}Switch sound instantly, e.g. oreo, blue, topre${RESET}
    ${GREEN}jaster volume${RESET}    ${GRAY}Show or set the volume: 60, up, down, mute${RESET}
    ${GREEN}jaster event${RESET}     ${GRAY}List detected keyboards${RESET}
    ${GREEN}jaster stop${RESET}      ${GRAY}Stop the Jaster daemon${RESET}
    ${GREEN}jaster update${RESET}    ${GRAY}Update to the latest version${RESET}

${PERMISSION}

${YELLOW}GitHub${RESET}  -   ${GRAY}https://github.com/JoeCelaster/Jaster${RESET}

              ${BOLD}${CYAN}Enjoy the typing experience!${RESET}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

EOF