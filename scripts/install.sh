#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/share/jaster

sudo cp "$SCRIPT_DIR/jaster" /usr/local/bin/jaster
sudo chmod +x /usr/local/bin/jaster

sudo cp -r "$SCRIPT_DIR/assets/sounds" /usr/share/jaster/

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

${YELLOW}Available Commands${RESET}

    ${GREEN}jaster doctor${RESET}    ${GRAY}Check installation${RESET}
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