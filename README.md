<img width="300" height="150" alt="Add a heading" src="https://github.com/user-attachments/assets/9ffaeb17-51a0-451b-a491-e9efd7126703" />

# Jaster

**Bring mechanical typing sounds to your native keyboard.**

Jaster is a lightweight CLI application that adds realistic mechanical typing sounds to any keyboard on Linux and Windows, providing an immersive typing experience with minimal setup.

---

> [!NOTE]
> **Current Platform Support**
>
> Jaster supports **Linux** and **Windows**.
>
> Support for **macOS** is under active development as part of Jaster's cross-platform roadmap.

## Installation

### Linux

Copy and paste the following commands into your terminal:

```bash
# Download and install Jaster
curl -fsSL https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.sh | bash

# Allow your user to access keyboard input devices
sudo usermod -aG input $USER

# Refresh your login session so the new permission takes effect
exec su - "$USER"

# Start Jaster and enable typing sounds
jaster start
```

### Windows

Paste this into PowerShell. No administrator rights are needed — Jaster
installs into your own user profile:

```powershell
irm https://raw.githubusercontent.com/JoeCelaster/Jaster/main/install.ps1 | iex
```

Then **open a new terminal** so the PATH change takes effect, and run:

```powershell
jaster start
```

That's it. Jaster is now running, and your keyboard will produce typing sounds.

---

## Commands

| Command | Description |
|---------|-------------|
| `jaster start` | Start Jaster and enable typing sounds. Asks which sound pack to use. |
| `jaster <sound>` | Switch to a sound pack by shortcut, e.g. `jaster oreo`. |
| `jaster sounds` | List the installed sound packs and their shortcuts. |
| `jaster volume` | Show or set how loud typing is. |
| `jaster stop` | Stop Jaster. |
| `jaster update` | Update Jaster to the latest version. |
| `jaster doctor` | Check installation, permissions, and system status. |
| `jaster event` | Display keyboard events detected by Jaster. |
| `jaster version` | Jaster's latest version. |

---

## Sound packs

Running `jaster start` asks which keyboard you want to sound like:

```
  Choose your keyboard sound (↑/↓ to move, enter to select)

  ❯ CherryMX Black - PBT keycaps       cherrymx-black-pbt
    CherryMX Blue - PBT keycaps        cherrymx-blue-pbt
    CherryMX Brown - PBT keycaps       cherrymx-brown-pbt
    CherryMX Red - PBT keycaps         cherrymx-red-pbt
    EG Crystal Purple                  eg-crystal-purple
    EG Oreo                            eg-oreo
    NK Cream                           nk-cream
    Topre Purple Hybrid - PBT keycaps  topre-purple-hybrid-pbt
```

Your choice is remembered, so the next `jaster start` defaults to it.

### Switching sounds

Every pack has a one-word shortcut. Type it to switch immediately — Jaster
restarts on the new sound, or starts if it wasn't running:

```bash
jaster oreo      # EG Oreo
jaster blue      # CherryMX Blue
jaster nkcream   # NK Cream
```

| Shortcut | Sound pack |
|----------|------------|
| `jaster black` | CherryMX Black - PBT keycaps |
| `jaster blue` | CherryMX Blue - PBT keycaps |
| `jaster brown` | CherryMX Brown - PBT keycaps |
| `jaster red` | CherryMX Red - PBT keycaps |
| `jaster crystal` | EG Crystal Purple |
| `jaster oreo` | EG Oreo |
| `jaster nkcream` | NK Cream |
| `jaster topre` | Topre Purple Hybrid - PBT keycaps |

Run `jaster sounds` any time to see the list with the current pack marked.

To pick a pack without the picker in a script or startup service, use the flag
form instead:

```bash
jaster start --sound topre
```

Both forms accept the shortcut, the full id, or any unambiguous fragment of
either. `jaster cherry` is rejected, since four packs match, and Jaster lists the
candidates instead of guessing.

Every pack is levelled to the same loudness when it loads, so switching sounds
changes the character of the typing, never the volume.

---

## Volume

**60 is best for headphones. 150 is best for speakers** — and 150 is where Jaster
starts, so headphone users want:

```bash
jaster volume 60
```

```
🔉 ▓▓▓░░░░░░░ 60%
```

The rest:

```bash
jaster volume        # show the current level
jaster volume up     # ±10 per step
jaster volume down
jaster volume mute   # same as 0
jaster volume max    # 200, the ceiling
```

Changes apply to a running Jaster within a moment — no restart, no need to stop
typing. `jaster vol` is a shorter alias. Loud settings stay clean: the peaks of
each keypress are eased down rather than allowed to clip.

To see what is installed:

```bash
jaster sounds
```

---

## How it works

Jaster listens for keyboard input events and plays synchronized typing sounds in real time. It is designed to be lightweight, responsive, and easy to install.

---

## Requirements

**Linux**

- Audio system supported by your distribution (ALSA/PipeWire/PulseAudio)
- Permission to access `/dev/input` devices

**Windows**

- Windows 10 or later
- Nothing else. Audio and key capture both use built-in Windows APIs, and the
  install needs no administrator rights.

---

## Troubleshooting

### Typing sounds are not working

Verify your installation:

```bash
jaster doctor
```

Check whether keyboard input is being detected:

```bash
jaster event
```

**On Linux**, if no keyboards appear, ensure your user has been added to the
`input` group and that you've refreshed your login session:

```bash
sudo usermod -aG input $USER
exec su - "$USER"
```

**On Windows**, two things are worth knowing:

- Anti-cheat software (Vanguard, EasyAntiCheat, BattlEye) and some endpoint
  security agents block low-level keyboard hooks. `jaster doctor` tells you
  whether the hook can be installed.
- Keys typed into windows running as administrator are silent unless Jaster is
  also running elevated. That is a Windows security boundary, not a bug.

The detached daemon writes its output to `%LOCALAPPDATA%\Jaster\daemon.log`,
which is the place to look if `jaster start` succeeds but nothing plays.

---

## Uninstall

**Linux** — remove the installed binary and its sounds:

```bash
sudo rm /usr/local/bin/jaster
sudo rm -rf /usr/share/jaster
rm -rf ~/.local/share/jaster
```

**Windows** — stop Jaster, then remove its folder and PATH entry:

```powershell
jaster stop
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Jaster"
```

Then remove `%LOCALAPPDATA%\Jaster` from your user PATH under
*Settings → System → About → Advanced system settings → Environment Variables*.

Verify removal by opening a new terminal and checking that `jaster` is no
longer found.

## Contributing

Contributions are always welcome.

If you find a bug, have a feature request, or want to improve Jaster, please open an issue or submit a pull request.
