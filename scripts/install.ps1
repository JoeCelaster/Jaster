#Requires -Version 5.1

# The bootstrap runs this as script text so a restrictive execution policy
# cannot block it, and that leaves $PSScriptRoot empty — so it passes the
# extracted directory in instead.
param([string]$Root)

$ErrorActionPreference = 'Stop'

$root = if ($Root) { $Root } elseif ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
$dest = Join-Path $env:LOCALAPPDATA 'Jaster'
$exe  = Join-Path $dest 'jaster.exe'

# `jaster update` renames the running binary aside before handing over to us, so
# a daemon started from it may report either name as its image.
$retired = Join-Path $dest 'jaster.old.exe'

# A running daemon holds jaster.exe open, so make sure nothing is holding the
# name we are about to write. This is also the one moment that can clear a
# daemon which lost track of its pid file and has been doubling every keystroke
# since.
if (Test-Path $exe) { & $exe stop 2>$null | Out-Null }

# `jaster update` runs this installer, so one of the jaster.exe processes on
# this machine is the update itself. Killing it mid-flight leaves the rename it
# did in place and no jaster.exe to replace it — the shell that was using the
# command a moment ago then cannot find it at all. It tells us its pid so we can
# step around it.
$spare = @($PID)

if ($env:JASTER_UPDATE_PID) { $spare += [int]$env:JASTER_UPDATE_PID }

foreach ($process in @(Get-Process -Name jaster -ErrorAction SilentlyContinue)) {
    if ($spare -contains $process.Id) { continue }

    # Reading the image path of a process we do not own throws, and a jaster
    # somewhere else on disk is not ours to stop either way.
    $image = $null
    try { $image = $process.Path } catch { }

    if ($image -eq $exe -or $image -eq $retired) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
}

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

# Compare whole entries, expanded, rather than looking for the text of $dest
# anywhere in the string: an entry may be stored as %LOCALAPPDATA%\Jaster, and
# a substring test would both miss that and match a directory that merely
# starts with the same name.
function Test-OnPath([string]$value, [string]$directory) {
    $wanted = $directory.TrimEnd('\')

    foreach ($entry in ($value -split ';')) {
        if (-not $entry) { continue }

        if ([Environment]::ExpandEnvironmentVariables($entry).Trim('"').TrimEnd('\') -eq $wanted) {
            return $true
        }
    }

    return $false
}

if (-not (Test-OnPath $current $dest)) {
    $updated = if ($current) { "$current;$dest" } else { $dest }
    Set-ItemProperty 'HKCU:\Environment' -Name Path -Value $updated -Type ExpandString

    # Explorer hands a cached copy of the environment to every terminal it
    # launches, and only refreshes it when told. Without this broadcast the new
    # PATH reaches nothing until the next sign-in, so even opening a fresh tab
    # would not find jaster.
    try {
        if (-not ('Jaster.Env' -as [type])) {
            Add-Type -Namespace Jaster -Name Env -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true, CharSet = System.Runtime.InteropServices.CharSet.Auto)]
public static extern System.IntPtr SendMessageTimeout(System.IntPtr hWnd, uint Msg,
    System.UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout,
    out System.UIntPtr lpdwResult);
'@
        }

        $answer = [UIntPtr]::Zero

        # HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG, 5s.
        [void][Jaster.Env]::SendMessageTimeout(
            [IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 0x2, 5000, [ref]$answer)
    } catch {
        Write-Host "Note: could not broadcast the PATH change; a new terminal will pick it up."
    }
}

# Usable in this session without reopening the terminal.
if (-not (Test-OnPath $env:Path $dest)) {
    $env:Path = "$env:Path;$dest"
}

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

    ${gray}Open a new terminal — a shell that was already running keeps${reset}
    ${gray}the PATH it started with.${reset}

${yellow}If typing is silent${reset}

    ${gray}Anti-cheat and endpoint security software can block the keyboard${reset}
    ${gray}hook. Keys typed into admin windows are silent unless Jaster runs${reset}
    ${gray}elevated too. Run ${reset}${green}jaster doctor${reset}${gray} to check.${reset}

${yellow}GitHub${reset}  -   ${gray}https://github.com/JoeCelaster/Jaster${reset}

              ${bold}${cyan}Enjoy the typing experience!${reset}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

"@
