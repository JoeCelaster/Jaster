# Contributor Setup Guide

This guide gets you from a fresh clone to a running development build of Jaster on
**Linux**, **macOS**, or **Windows**.

---

## Read this first: the current platform reality

Jaster's runtime is **Linux-only today**, and that is enforced at the dependency
level, not just at runtime:

| Crate | Why it is Linux-only | Declared in `Cargo.toml` as |
|-------|----------------------|------------------------------|
| `evdev` | Wraps the Linux input subsystem (`/dev/input`) via `nix` ioctls | unconditional dependency |
| `udev`  | Binds `libudev-sys`, resolved through `pkg-config` | unconditional dependency |

Because both are unconditional, **`cargo build` fails on macOS and Windows before
any Jaster code is even compiled.** This is not a bug you have caused — it is the
main thing the cross-platform port has to fix.

What this means for you:

- **Linux contributors** can build, run, and test everything natively.
- **macOS and Windows contributors** can still contribute today, but you need a
  Linux environment (VM, WSL2, or container) to compile and run. See
  [macOS](#macos) and [Windows](#windows) below.
- The cross-platform port itself is tracked in
  [Porting roadmap](#porting-roadmap-what-cross-platform-work-actually-needs) —
  that section is the highest-value place to contribute right now.

The good news: `rdev`, which is *already* a dependency and already wrapped in
`src/keyboard/hook.rs`, is genuinely cross-platform (X11 on Linux, CoreGraphics on
macOS, Win32 on Windows). The port has a clear path.

---

## Prerequisites (all platforms)

| Tool | Minimum | Notes |
|------|---------|-------|
| Rust | **1.85** | The crate uses `edition = "2024"`, which requires 1.85+. Developed against 1.97. |
| Cargo | ships with Rust | |
| Git | any recent | |

Install Rust via [rustup](https://rustup.rs) on every platform:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run [`rustup-init.exe`](https://rustup.rs) instead.

Verify:

```bash
rustc --version   # must be >= 1.85.0
cargo --version
```

Clone the repository:

```bash
git clone https://github.com/JoeCelaster/Jaster.git
cd Jaster
```

---

## Linux

This is the fully supported path. Everything below works natively.

### 1. Install system dependencies

Jaster links against ALSA (audio), libudev (device enumeration), and X11 +
XTest + XInput (used by the `rdev` hook layer). These are the same packages the
release CI installs.

**Debian / Ubuntu / Pop!\_OS / Mint**

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libasound2-dev \
  libudev-dev \
  libx11-dev \
  libxtst-dev \
  libxi-dev \
  libxrandr-dev
```

**Fedora / RHEL**

```bash
sudo dnf install -y \
  gcc \
  pkgconf-pkg-config \
  alsa-lib-devel \
  systemd-devel \
  libX11-devel \
  libXtst-devel \
  libXi-devel \
  libXrandr-devel
```

**Arch / Manjaro**

```bash
sudo pacman -S --needed \
  base-devel \
  pkgconf \
  alsa-lib \
  systemd-libs \
  libx11 \
  libxtst \
  libxi \
  libxrandr
```

**openSUSE**

```bash
sudo zypper install -y \
  gcc \
  pkg-config \
  alsa-devel \
  systemd-devel \
  libX11-devel \
  libXtst-devel \
  libXi-devel \
  libXrandr-devel
```

### 2. Grant keyboard access

Jaster reads raw key events from `/dev/input/event*`, which requires membership
in the `input` group:

```bash
sudo usermod -aG input $USER
```

Group membership only applies to **new** login sessions. Either log out and back
in, or refresh the current shell:

```bash
exec su - "$USER"
```

Confirm it took effect:

```bash
groups | grep input
```

Without this, `cargo run -- doctor` reports a permission error and the daemon
finds zero keyboards.

### 3. Build and run

```bash
cargo build
cargo run -- doctor    # environment check: OS, audio backend, keyboards
cargo run -- event     # list detected keyboards
cargo run -- start     # start the background daemon
cargo run -- stop      # stop it
```

> **Run from the repository root.** `src/audio/cache.rs` resolves sounds from
> `assets/sounds` relative to the current working directory, falling back to the
> installed path `/usr/share/jaster/sounds`. From any other directory, a dev build
> silently loads the *installed* sounds instead of the ones in your working tree.

### A note on Wayland

Device reading through `evdev` works on both X11 and Wayland, so `jaster start`
is fine either way. The `rdev`-based hook in `src/keyboard/hook.rs` is X11-only
and will not capture events under a pure Wayland session — worth knowing if you
start working on that layer.

---

## macOS

**Native `cargo build` does not work yet** — it fails while compiling `evdev` and
`udev`. Choose one of the two paths below.

### Path A — contribute via a Linux environment (recommended for now)

Use a Linux VM or container to build and run, and edit code natively on macOS.

**Docker / Podman** (works on Apple Silicon and Intel):

```bash
docker run --rm -it \
  -v "$PWD":/work -w /work \
  rust:1.97 \
  bash -c "apt-get update && \
           apt-get install -y pkg-config libasound2-dev libudev-dev \
             libx11-dev libxtst-dev libxi-dev libxrandr-dev && \
           cargo build"
```

This gives you a compiling build for verifying code changes, `cargo clippy`, and
`cargo fmt`. It does **not** give you real keyboard or audio hardware, so
`jaster start` cannot be end-to-end tested in a container — a full VM (UTM,
Parallels, VMware Fusion, or OrbStack with a desktop image) with USB passthrough
is needed for that.

**Practical split:** verify compilation and lints in the container, and ask a
Linux maintainer to smoke-test hardware behavior on the PR.

### Path B — work on the macOS port itself

This is the contribution that removes the need for Path A. Install the host
toolchain you will need:

```bash
xcode-select --install          # Apple's command line tools (clang, linker)
brew install pkg-config         # if you do not already have it
```

macOS needs no extra audio packages — `rodio` targets CoreAudio directly, and
`rdev` targets CoreGraphics. Once `evdev`/`udev` are properly gated (see
[Porting roadmap](#porting-roadmap-what-cross-platform-work-actually-needs)), the
crate should compile natively with just the above.

**macOS permissions.** Any global key listener on macOS requires the user to
grant your terminal (or the built binary) **Input Monitoring** and, for some
APIs, **Accessibility** access:

`System Settings → Privacy & Security → Input Monitoring` (and `→ Accessibility`)

Without it, `rdev::listen` returns an error or silently receives no events. The
macOS port will need to detect this state and surface it in `jaster doctor`, the
same way the Linux path surfaces the `input` group requirement.

---

## Windows

**Native `cargo build` does not work yet** — same `evdev` / `udev` failure as
macOS. Choose one of the two paths below.

### Path A — contribute via WSL2 (recommended for now)

```powershell
wsl --install -d Ubuntu
```

Then, **inside** the WSL2 shell, follow the entire [Linux](#linux) section
(system packages, `input` group, build).

Important WSL2 caveats:

- **Clone inside the WSL filesystem** (e.g. `~/Jaster`), not under `/mnt/c/...`.
  Cargo builds across the Windows/Linux filesystem boundary are dramatically
  slower and can hit file-locking issues.
- **WSL2 has no access to your physical keyboard's `/dev/input` devices.**
  `jaster event` will find nothing, and `jaster start` will exit with
  "No keyboards found." This is expected — WSL2 does not pass through host HID
  devices.
- WSLg provides audio, so audio-side changes can be checked, but full end-to-end
  behavior needs real Linux hardware or a VM with USB passthrough.

So WSL2 is excellent for **compiling, linting, and reviewing code**, and not
suitable for **hardware verification**. Note that limitation in your PR and a
Linux maintainer can confirm runtime behavior.

### Path B — work on the Windows port itself

Install the native build toolchain:

1. [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
   with the **"Desktop development with C++"** workload (provides the MSVC
   linker that Rust's default `x86_64-pc-windows-msvc` target requires).
2. Rust via [`rustup-init.exe`](https://rustup.rs).

No extra audio libraries are needed — `rodio` uses WASAPI and `rdev` uses the
Win32 low-level keyboard hook, both part of the OS. Once `evdev`/`udev` are
gated behind `cfg(target_os = "linux")`, the crate should build natively.

Windows generally does **not** require a special permission grant for
`SetWindowsHookEx`-based key listening, but some anti-cheat and endpoint
security software will block or flag it — worth documenting in the port.

---

## Everyday development commands

```bash
cargo build              # debug build
cargo build --release    # optimized build, matches what CI ships
cargo run -- <command>   # run a subcommand from source
cargo fmt                # format (run before every commit)
cargo clippy -- -D warnings   # lint
cargo test               # test suite
```

Available subcommands: `start`, `stop`, `doctor`, `event`, `update`, `version`
(plus a hidden `daemon`, which `start` spawns for you — you rarely invoke it
directly, though running `cargo run -- daemon` in the foreground is the easiest
way to watch key events and audio errors live).

### Two things to avoid in a dev checkout

1. **Do not run `cargo run -- update`.** That command stops the running daemon,
   downloads the latest *released* binary from GitHub, and runs the system
   installer — overwriting `/usr/local/bin/jaster` and `/usr/share/jaster`. It is
   an end-user command, not a developer one.

2. **Watch for a stale PID file.** `start` writes the daemon PID to
   `~/.local/share/jaster/jaster.pid`, and `stop` reads it back. If a daemon dies
   without cleanup, `stop` reports success while doing nothing useful. Clear it
   manually if the state gets confusing:

   ```bash
   pkill -f 'jaster daemon'
   rm -f ~/.local/share/jaster/jaster.pid
   ```

---

## Source layout

```
src/
├── main.rs               entrypoint; dispatches subcommands
├── lib.rs                module root
├── cli/args.rs           clap CLI definition
├── commands/             one file per subcommand
│   ├── start.rs          spawns the detached daemon (setsid), writes the PID file
│   ├── daemon.rs         the real work: audio engine + sound cache + one thread per keyboard
│   ├── stop.rs           reads the PID file, kills the daemon
│   ├── doctor.rs         environment diagnostics
│   ├── sounds.rs         lists the installed sound packs
│   ├── switch.rs         `jaster oreo` — clap external subcommand; restarts the daemon
│   ├── volume.rs         `jaster volume` — shows/sets the saved level
│   ├── event.rs          lists detected keyboards
│   ├── update.rs         self-update via the GitHub release installer
│   └── version.rs        prints CARGO_PKG_VERSION
├── keyboard/
│   ├── discovery.rs      scans /dev/input for devices with A + ENTER + SPACE  [Linux-only]
│   └── hook.rs           rdev-based listener wrapper                    [cross-platform, currently unused]
├── audio/
│   ├── engine.rs         rodio output stream
│   ├── theme.rs          discovers sound packs and parses their config.json
│   ├── cache.rs          decodes a pack into one in-memory clip per key, levelled
│   ├── volume.rs         the saved level, plus the daemon's live view of it
│   └── player.rs         plays a buffered sound into the mixer
├── utils/select.rs       raw-mode arrow-key picker used by `jaster start`
└── utils/pid.rs          PID file helpers (not currently wired into `utils/mod.rs`)
```

Empty stub files exist at `src/commands/init.rs`, `src/keyboard/events.rs`, and
`src/keyboard/mapper.rs` — they are placeholders, not modules in use.

### Sound packs

Sound assets live in `assets/sounds/<pack>/`, each with a Mechvibes-style
`config.json`. Any directory containing a `config.json` is picked up
automatically — no code change is needed to add a pack.

Two pack layouts are supported, and `src/audio/cache.rs` handles both:

| `key_define_type` | Layout | Example |
|-------------------|--------|---------|
| `multi` | one file per key, `defines` maps a keycode to a filename | `nk-cream` |
| `single` | one sound sheet, `defines` maps a keycode to `[offset_ms, duration_ms]` | the four `cherrymx-*-pbt` packs, `eg-oreo`, `eg-crystal-purple`, `topre-purple-hybrid-pbt` |

The sheet filename comes from the config's `sound` field, so it does not have to
be `sound.ogg` (`eg-crystal-purple` uses `purple.ogg`). Dropping a new pack
directory into `assets/sounds/` is all it takes; `tests/sound_packs.rs` then
covers it automatically.

Pack keycodes are PS/2 set 1 scancodes, which match Linux evdev keycodes for the
main key block; extended keys are stored as `0xE00 + low byte` and are translated
by a small table in `src/audio/theme.rs`. Codes outside both ranges are skipped
and fall back to the generic click.

`single` packs are decoded once, then sliced per key with a short fade in/out so
the cuts do not click. Clips that repeat (a shared file or a shared sheet offset)
are decoded once and shared via `SamplesBuffer`'s internal `Arc`.

The selected pack is remembered in `~/.local/share/jaster/sound-pack` and can be
overridden with `jaster start --sound <pack>`; `nk-cream` is the default when
nothing has been chosen yet.

`theme::find` resolves a pack from an id, a display name, a one-word shortcut, or
an unambiguous fragment, comparing on a normalized form (lowercase, alphanumerics
only) so `NK Cream`, `nk-cream` and `nkcream` are the same string. It reports the
candidates rather than guessing when more than one pack matches.

### Loudness and volume

The packs are recorded at wildly different levels — as shipped they span about
16 dB, from `nk-cream` at -43 dBFS to `cherrymx-blue-pbt` at -27 dBFS — so
switching packs used to mean lunging for the system volume.

`cache.rs` therefore measures a pack once it is decoded and applies a single
gain: `min(TARGET_RMS / rms, PEAK_CEILING / peak)`. One factor for the whole
pack, so a pack's own loud keys (space, enter) stay louder than its letters —
only the pack-to-pack difference is removed. The peak term is what stops a quiet
pack being amplified into clipping; `eg-oreo` is the one pack that hits it.
`tests/sound_packs.rs` fails if the packs drift more than 2 dB apart or if any
peak reaches clipping, so a newly added pack cannot land louder than the rest.

User volume is separate and multiplies on top, in `player.rs`. The level lives in
`~/.local/share/jaster/volume` as a plain percentage, where 100 is the normalized
level. There is no IPC to the daemon, so `audio::volume::Level` polls that file
every 400ms into an `AtomicU32` the keyboard threads read per keypress — that is
why `jaster volume` takes effect without a restart.

The default is **150** (`volume::SPEAKERS`), which is deliberately above the
normalized level: laptop speakers need it, and headphone users are told to drop
to 60 (`volume::HEADPHONES`) by `jaster start`, `jaster update`, `jaster volume`
and the installer, all of which print `volume::advice()`.

Because the default is above 100, gain alone would push the loudest transients
past 1.0 and hard-clip them, which on a keypress sounds like a crackle.
`player.rs` therefore ends the chain with a soft limiter: samples below 0.8 pass
through untouched — ordinary typing is unaffected — and above the knee the curve
eases asymptotically towards 1.0, so no volume in range can clip. `soften()` is
public so `tests/sound_packs.rs` can check the curve at the worst case Jaster can
actually produce (a pack at the peak ceiling, played at `volume::MAX`).

**Adding a pack** means dropping the directory into `assets/sounds/` and adding
one line to `theme::SHORTCUTS` so it gets a `jaster <word>` shortcut.
`tests/sound_packs.rs` fails if an installed pack has no shortcut, if two
shortcuts collide, or if a shortcut shadows a real subcommand (which clap would
route to the command, never to the switch).

---

## Porting roadmap: what cross-platform work actually needs

If you want to make Jaster build and run on your own OS, this is the work. Each
item is independently reviewable, so feel free to take just one.

**1. Gate the Linux-only dependencies.** In `Cargo.toml`, move `evdev` and `udev`
under a target-specific table so non-Linux builds stop failing:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
evdev = "0.13"
udev = "0.9.3"
libc = "0.2.189"
```

**2. Drop the unused `udev` dependency.** Nothing in `src/` imports it. Removing
it eliminates one Linux-only, pkg-config-dependent crate outright.

**3. Introduce a platform-neutral key type.** `evdev::KeyCode` currently leaks
into shared code — `src/audio/cache.rs` keys its `HashMap` on it, even though
sound caching is not inherently Linux-specific. Define a Jaster-owned `Key` enum
and convert at the platform boundary (`evdev::KeyCode → Key` on Linux,
`rdev::Key → Key` elsewhere).

**4. Add a backend abstraction for key capture.** Something like a
`KeyListener` trait with an evdev implementation on Linux and an `rdev`
implementation on macOS/Windows. `src/keyboard/hook.rs` is already the rdev
wrapper — it just needs to be wired into `daemon.rs` behind a `cfg`.

**5. Make paths cross-platform.** Several places assume Unix layout:
`src/utils/pid.rs` and `src/commands/start.rs`/`stop.rs` read `$HOME` and use
`~/.local/share/jaster`, and `src/audio/cache.rs` falls back to
`/usr/share/jaster/sounds`. Windows has neither. A crate like `directories`, or
a small `cfg`-gated helper, resolves this.

**6. Replace shelling out with native calls.** `stop.rs` runs the `kill` binary
and `doctor.rs` runs `sh -c` — neither exists on Windows.

**7. Extend `doctor` per platform.** It currently early-returns on any non-Linux
OS. Each platform needs its own checks: Input Monitoring permission on macOS,
audio device availability on Windows, in place of the Linux `input`-group check.

**8. Extend the release CI.** `.github/workflows` builds only
`jaster-linux-x86_64.tar.gz`. macOS and Windows need their own build jobs and
their own install/uninstall paths.

---

## Contribution workflow

1. Fork the repository and create a branch:

   ```bash
   git checkout -b feat/macos-key-listener
   ```

2. Make your change. Before committing:

   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo build
   cargo test
   ```

3. Write a clear commit message describing what changed and why.

4. Open a pull request against `main`. In the description, state:
   - which OS you developed and tested on;
   - whether you were able to verify runtime behavior on real hardware, or only
     that it compiles (entirely fine — just say so, so a maintainer knows to
     smoke-test it).

Note that CI currently runs only on `v*` tags to publish releases; there is no
automated check on pull requests yet. Run the commands above locally — a
reviewer is relying on you having done so.

### Version bumps

The release version lives in `Cargo.toml` and must be committed together with
the updated `Cargo.lock` — several past commits exist purely to repair a
mismatch between the two.

Also be aware of an existing inconsistency worth fixing if you touch this area:
`src/cli/args.rs` hardcodes `#[command(version = "0.1.0")]`, so `jaster --version`
reports `0.1.0` while `jaster version` correctly reports the `Cargo.toml`
version. Replacing the literal with `env!("CARGO_PKG_VERSION")` fixes it.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `cargo build` fails compiling `evdev` or `udev` on macOS/Windows | Linux-only deps are unconditional | Expected today — use WSL2 / a VM, or take on the port |
| `error: failed to run custom build command for libudev-sys` | Missing libudev headers | Install `libudev-dev` / `systemd-devel` |
| `ALSA lib ... cannot find card` or link error on `-lasound` | Missing ALSA headers | Install `libasound2-dev` / `alsa-lib-devel` |
| Link error mentioning `X11`, `Xtst`, or `Xi` | Missing X11 dev packages | Install `libx11-dev libxtst-dev libxi-dev libxrandr-dev` |
| `jaster doctor` reports "Permission denied" | User is not in the `input` group | `sudo usermod -aG input $USER`, then start a new session |
| No keyboards detected, permissions look correct | Session not refreshed since the group change | `exec su - "$USER"`, or log out and back in |
| No keyboards detected inside WSL2 | WSL2 does not expose host HID devices | Expected — verify on real Linux hardware or a VM |
| Daemon runs but no sound | Wrong working directory, so a dev build loaded installed assets | Run `cargo run` from the repository root |
| `jaster stop` says "stopped" but sound continues | Stale PID file | `pkill -f 'jaster daemon'` and remove `~/.local/share/jaster/jaster.pid` |

---

## Getting help

- Open an issue: <https://github.com/JoeCelaster/Jaster/issues>
- Run `cargo run -- doctor` and paste the output — it is the fastest way for a
  maintainer to see your environment.
