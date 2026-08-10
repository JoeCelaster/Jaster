<img width="300" height="150" alt="Add a heading" src="https://github.com/user-attachments/assets/9ffaeb17-51a0-451b-a491-e9efd7126703" />

# Jaster

**Bring mechanical typing sounds to your native keyboard.**

Jaster is a lightweight CLI application that adds realistic mechanical typing sounds to any keyboard on Linux, providing an immersive typing experience with minimal setup.

---

> [!NOTE]
> **Current Platform Support**
>
> Jaster currently supports **Linux-based distributions** only.
>
> Support for **Windows** and **macOS** is under active development as part of Jaster's cross-platform roadmap.

## Installation

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

That's it. Jaster is now running, and your keyboard will produce typing sounds.

---

## Commands

| Command | Description |
|---------|-------------|
| `jaster start` | Start Jaster and enable typing sounds. |
| `jaster stop` | Stop Jaster. |
| `jaster doctor` | Check installation, permissions, and system status. |
| `jaster event` | Display keyboard events detected by Jaster. |

---

## How it works

Jaster listens for keyboard input events and plays synchronized typing sounds in real time. It is designed to be lightweight, responsive, and easy to install.

---

## Requirements

- Linux
- Audio system supported by your distribution (ALSA/PipeWire/PulseAudio)
- Permission to access `/dev/input` devices

---

## Troubleshooting

### Typing sounds are not working

Verify your installation:

```bash
jaster doctor
```

Check whether keyboard events are being detected:

```bash
jaster event
```

If no events appear, ensure your user has been added to the `input` group and that you've refreshed your login session:

```bash
sudo usermod -aG input $USER
exec su - "$USER"
```

---

## Uninstall

Remove the installed binary:

```bash
sudo rm /usr/local/bin/jaster
```

Verify removal:

```bash
which jaster
```

If nothing is returned, Jaster has been removed successfully.

## Contributing

Contributions are always welcome.

If you find a bug, have a feature request, or want to improve Jaster, please open an issue or submit a pull request.
