#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/share/jaster

# Staged, then renamed into place: `jaster update` runs this script from the
# very binary we are replacing, and writing over a running executable fails
# with "Text file busy". A rename swaps the file instead of rewriting it.
sudo cp "$SCRIPT_DIR/jaster" /usr/local/bin/jaster.new
sudo chmod +x /usr/local/bin/jaster.new
sudo mv -f /usr/local/bin/jaster.new /usr/local/bin/jaster

sudo cp -r "$SCRIPT_DIR/assets/sounds" /usr/share/jaster/

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

${YELLOW}If no keyboards are detected${RESET}

    sudo usermod -aG input \$USER
    exec su - "\$USER"

${YELLOW}GitHub${RESET}  -   ${GRAY}https://github.com/JoeCelaster/Jaster${RESET}

              ${BOLD}${CYAN}Enjoy the typing experience!${RESET}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

EOF