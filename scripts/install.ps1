#Requires -Version 5.1

# The bootstrap runs this as script text so a restrictive execution policy
# cannot block it, and that leaves $PSScriptRoot empty — so it passes the
# extracted directory in instead.
param([string]$Root)

$ErrorActionPreference = 'Stop'

$root = if ($Root) { $Root } elseif ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
$dest = Join-Path $env:LOCALAPPDATA 'Jaster'

# A running daemon holds jaster.exe open, and `jaster update` renames the
# binary out of the way before calling us — either way, make sure nothing is
# holding the name we are about to write.
if (Test-Path (Join-Path $dest 'jaster.exe')) {
    & (Join-Path $dest 'jaster.exe') stop 2>$null | Out-Null
}
Get-Process jaster -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300

New-Item -ItemType Directory -Force -Path $dest | Out-Null

Copy-Item (Join-Path $root 'jaster.exe') (Join-Path $dest 'jaster.exe') -Force
Copy-Item (Join-Path $root 'assets\sounds') $dest -Recurse -Force

# Add to the user PATH. Read and write the registry directly rather than using
# [Environment]::SetEnvironmentVariable: that reads REG_EXPAND_SZ values
# already expanded and writes them back as plain strings, permanently baking
# whatever %USERPROFILE% happened to be into the user's PATH.
$current = (Get-Item 'HKCU:\Environment').GetValue(
    'Path', '', 'DoNotExpandEnvironmentNames')

if ($current -notlike "*$dest*") {
    $updated = if ($current) { "$current;$dest" } else { $dest }
    Set-ItemProperty 'HKCU:\Environment' -Name Path -Value $updated -Type ExpandString
}

# Usable in this session without reopening the terminal.
$env:Path = "$env:Path;$dest"

# `jaster update` sets this. It reports the new version and restarts the daemon
# itself, so the first-run welcome below would only be noise on top of that.
if ($env:JASTER_UPDATE) { exit 0 }

$e = [char]27
$green  = "$e[32m"; $cyan = "$e[36m"; $yellow = "$e[33m"
$gray   = "$e[90m"; $bold = "$e[1m";  $reset  = "$e[0m"

Write-Host @"

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

                  ${bold}${cyan} Jaster is Ready! ${reset}

${yellow}Get Started${reset}

    ${green}jaster start${reset}

${yellow}Volume${reset}   -   ${gray}starts at 150${reset}

    ${gray}60 for headphones${reset}   ${green}jaster volume 60${reset}
    ${gray}150 for speakers${reset}    ${green}jaster volume 150${reset}

${yellow}Available Commands${reset}

    ${yellow}jaster doctor${reset}    ${gray}Check installation${reset}
    ${green}jaster sounds${reset}    ${gray}List installed sound packs and their shortcuts${reset}
    ${green}jaster oreo${reset}      ${gray}Switch sound instantly, e.g. oreo, blue, topre${reset}
    ${green}jaster volume${reset}    ${gray}Show or set the volume: 60, up, down, mute${reset}
    ${green}jaster event${reset}     ${gray}List detected keyboards${reset}
    ${green}jaster stop${reset}      ${gray}Stop the Jaster daemon${reset}
    ${green}jaster update${reset}    ${gray}Update to the latest version${reset}

${yellow}If jaster is not found${reset}

    ${gray}Open a new terminal — PATH changes need a fresh shell.${reset}

${yellow}If typing is silent${reset}

    ${gray}Anti-cheat and endpoint security software can block the keyboard${reset}
    ${gray}hook. Keys typed into admin windows are silent unless Jaster runs${reset}
    ${gray}elevated too. Run ${reset}${green}jaster doctor${reset}${gray} to check.${reset}

${yellow}GitHub${reset}  -   ${gray}https://github.com/JoeCelaster/Jaster${reset}

              ${bold}${cyan}Enjoy the typing experience!${reset}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

"@
