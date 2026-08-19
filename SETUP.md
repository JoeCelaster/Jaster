# Contributor Setup Guide

This guide gets you from a fresh clone to a running development build of Jaster on
**Linux**, **macOS**, or **Windows**.

---

## Read this first: the current platform reality

**Linux, macOS and Windows are all first-class targets.** Each builds and runs
natively, and CI checks all three on every push.

Platform-specific code lives in exactly two places, and the rest of the
codebase is free of `#[cfg]`:

| Concern | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Key capture | `src/keyboard/linux.rs` — `evdev`, one thread per `/dev/input` device | `src/keyboard/macos.rs` — a `CGEventTap` and its run loop | `src/keyboard/windows.rs` — a `WH_KEYBOARD_LL` hook and its message pump |
| Paths, process control, console | `#[cfg(unix)]` arms in `src/utils/` and `src/commands/`, plus a few `#[cfg(target_os = "linux")]` ones | the same `#[cfg(unix)]` arms, plus `#[cfg(target_os = "macos")]` where `/proc` and `/usr/share` do not exist | `#[cfg(windows)]` arms alongside them |

`src/keyboard/mod.rs` picks the backend with `#[cfg_attr(..., path = ...)]`, so
all three must export the same `listen` and `sources`; a symbol missing from one
is a compile error rather than a silent gap.

Everything else already travels. `rodio` selects ALSA on Linux, CoreAudio on
macOS and WASAPI on Windows automatically, and the decode/slice/normalize/limit
pipeline in `src/audio/` has no OS dependency at all.

**Watch the `unix` / `linux` distinction.** macOS is Unix, so a `#[cfg(unix)]`
arm written with only Linux in mind compiles there and is wrong at runtime —
which is silent. `is_jaster` in `src/utils/pid.rs` is the worked example: the
`/proc/<pid>/comm` read it used to do under `#[cfg(unix)]` would have answered
"not Jaster" for every pid on macOS, so `jaster stop` would have reported
success while the daemon played on.

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

Jaster links against ALSA for audio. That is the only system library it needs —
`evdev` is pure Rust, and the X11 packages that used to be required went away
with the `rdev` dependency. These are the same packages the release CI installs.

**Debian / Ubuntu / Pop!\_OS / Mint**

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libasound2-dev
```

**Fedora / RHEL**

```bash
sudo dnf install -y \
  gcc \
  pkgconf-pkg-config \
  alsa-lib-devel
```

**Arch / Manjaro**

```bash
sudo pacman -S --needed \
  base-devel \
  pkgconf \
  alsa-lib
```

**openSUSE**

```bash
sudo zypper install -y \
  gcc \
  pkg-config \
  alsa-devel
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
> Set `JASTER_SOUNDS=/path/to/assets/sounds` to pin it explicitly — see
> `sound_root()` in `src/utils/paths.rs` for the full resolution order.

### A note on Wayland

Jaster reads devices through `evdev`, below the display server, so `jaster start`
works identically on X11 and Wayland. There is no X11 dependency anywhere in the
Linux backend.

---

## macOS

macOS builds and runs natively. Install Apple's command line tools, which is the
whole toolchain requirement:

```bash
xcode-select --install    # clang and the linker
```

There is nothing else to install. CoreAudio, CoreGraphics and IOKit ship with
the OS, `rodio` targets CoreAudio directly, and `src/keyboard/macos.rs` declares
the framework calls it needs itself rather than pulling in a wrapper crate — the
same choice `windows.rs` makes with windows-sys.

```bash
cargo build
cargo test
cargo run -- doctor
```

macOS 10.15 (Catalina) is the floor: the permission calls the backend uses,
`CGPreflightListenEventAccess` and `IOHIDCheckAccess`, do not exist before it.

### Input Monitoring, and why it is granted to your terminal

A `CGEventTap` receives nothing until the user grants **Input Monitoring** under
*System Settings → Privacy & Security → Input Monitoring*. Four things about it
cost time if you learn them by debugging instead of reading:

- **The grant belongs to whatever *launched* Jaster**, not to Jaster. Run from a
  terminal, the switch to turn on carries your terminal's name — Terminal,
  iTerm2, Ghostty, VS Code — and there may be no "jaster" entry in the list at
  all. Each terminal you develop from needs its own.
- **Only processes started after the grant see it.** Quit the terminal with ⌘Q
  and reopen it; a new window or tab is not enough.
- **The grant is tied to the exact binary that asked**, which is why users have
  to allow Jaster once more after `jaster update` replaces it. Launching from a
  terminal you have already granted, your own rebuilds are covered by the
  terminal's grant.
- **Missing permission is not an error, it is a null port.** Since 10.15,
  `CGEventTapCreate` returns null rather than a tap that never fires, which is
  what `listen()` turns into a real error message.

`cargo run -- doctor` reports all of this, and will trigger the system prompt
itself when nothing has asked yet — there is no entry in System Settings to
point anyone at until something requests it.

**Secure Keyboard Entry.** Typing is silent inside password fields and in any app
that turns on Secure Input (Terminal has it in its own menu). macOS shuts every
event tap out of those by design. If keys stop making a sound in one app only,
that is this and not a bug.

### Things to know when working on the macOS backend

`src/keyboard/macos.rs` is a run loop with a callback, so it shares most of its
hazards with the Windows hook rather than with the evdev threads:

- **The system switches the tap off** when a callback is slow
  (`kCGEventTapDisabledByTimeout`) or when the user's input outruns it
  (`...ByUserInput`), and reports it nowhere else — the daemon goes deaf while
  still looking healthy. `handle` re-arms the tap from inside the callback,
  which is what the `port` `Cell` on `Tap` exists for. This is the macOS twin of
  the Windows 300 ms `LowLevelHooksTimeout`.
- **The callback does a table lookup and a `try_send`, nothing else.** No audio,
  no allocation, no lock the audio side might be holding. The consumer thread in
  `listen()` does the work.
- **The tap is listen-only** (`kCGEventTapOptionListenOnly`), so it cannot
  swallow a keystroke even by mistake — the OS enforces the invariant
  `windows.rs` has to uphold by hand with `CallNextHookEx`.
- **It is a session tap, not a HID tap.** `kCGHIDEventTap` wants root; the
  session level sees every key in the login session without it.
- **Auto-repeat comes marked**, like evdev's `value == 2`, so there is no
  held-key table to maintain — unlike Windows.
- **Modifiers arrive as `kCGEventFlagsChanged`**, which fires on both press and
  release and says which it was nowhere. The flag bit only means "something in
  this group is down", so left shift and right shift are indistinguishable by
  flags alone. `transition()` tracks each key and uses the group bit to
  resynchronise.

**The half you can test anywhere.** The keycode table and that modifier logic
live in `src/keyboard/macos_keys.rs`, outside the backend, compiled under
`#[cfg(any(target_os = "macos", test))]`. They are pure arithmetic, they are the
part that fails silently — a wrong scancode does not crash, it plays the generic
click forever — and they are covered by `cargo test` on every runner, Linux and
Windows included. Anything you can express without a framework call belongs
there rather than in `macos.rs`.

### Type-checking macOS from Linux or Windows

Most breakage is reachable without a Mac. Type-checking needs no macOS SDK:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo clippy --target aarch64-apple-darwin --all-targets -- -D warnings
cargo clippy --target x86_64-apple-darwin --all-targets -- -D warnings
```

Run this before pushing anything that touches `#[cfg]`-gated code; CI does the
same in the `macos-cross-check` job. Its limit is worth knowing: there is no
linker step, so it finds bad Rust but not a misspelled framework symbol. Only
the real `macos-latest` job links the binary, which is why `cargo test` runs
there.

### Contributing from macOS to the Linux side

If you need to verify Linux behavior, a container gets you compiling and linting
(Apple Silicon and Intel alike):

```bash
docker run --rm -it \
  -v "$PWD":/work -w /work \
  rust:1.97 \
  bash -c "apt-get update && \
           apt-get install -y pkg-config libasound2-dev && \
           cargo build"
```

It does **not** give you real keyboard or audio hardware, so `jaster start`
cannot be tested end to end in a container — a full VM (UTM, Parallels, VMware
Fusion, or OrbStack with a desktop image) with USB passthrough is needed for
that. In practice: verify compilation and lints in the container, and say on the
PR that a Linux maintainer should smoke-test the hardware behavior.

---

## Windows

Windows builds and runs natively. Install the toolchain:

1. [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
   with the **"Desktop development with C++"** workload (provides the MSVC
   linker that Rust's default `x86_64-pc-windows-msvc` target requires).
2. Rust via [`rustup-init.exe`](https://rustup.rs).

No extra libraries are needed — `rodio` uses WASAPI and key capture uses the
Win32 low-level keyboard hook, both part of the OS.

```powershell
cargo build
cargo test
.\target\debug\jaster.exe doctor
```

### Things to know when working on the Windows backend

`src/keyboard/windows.rs` is small, but three of its constraints are not
obvious and all three cause bugs that look like something else:

- **`CallNextHookEx` must always be called and its result returned.** Returning
  anything else swallows the keystroke for *every application on the desktop*.
  Test by typing into Notepad while the daemon runs.
- **The hook proc has roughly 300 ms** (`LowLevelHooksTimeout`) per call. Exceed
  it and Windows silently stops calling you — the daemon appears to work and
  then goes deaf. So the proc only does bookkeeping and a non-blocking
  `try_send`; audio happens on a consumer thread.
- **Auto-repeat is not filtered for you.** Unlike evdev's `value == 2`, a held
  key produces ordinary repeat `WM_KEYDOWN` messages, so the backend tracks
  which keys are physically down. Without that, holding a key machine-guns.

Two more worth knowing when testing:

- A low-level hook does **not** see keys typed into higher-integrity windows
  (anything running as administrator, UAC prompts, the secure desktop). Those
  are silent unless Jaster is elevated too.
- Anti-cheat and endpoint security software may block the hook entirely.
  `jaster doctor` probes this by installing and immediately releasing one.

Because `jaster start` detaches the daemon with `DETACHED_PROCESS`, it has no
console — its output goes to `%LOCALAPPDATA%\Jaster\daemon.log`. Check there
first when the daemon starts but nothing plays.

### Type-checking Windows from Linux

You do not need a Windows machine to catch most breakage. Type-checking needs
no MSVC linker:

```bash
rustup target add x86_64-pc-windows-msvc
cargo check --target x86_64-pc-windows-msvc
cargo clippy --target x86_64-pc-windows-msvc --all-targets
```

Run this before pushing anything that touches `#[cfg]`-gated code. Full builds
and any actual hook testing still need Windows; CI covers the former.

### Contributing from Windows to the Linux side

If you need to verify Linux behavior, WSL2 works for compiling and linting:

```powershell
wsl --install -d Ubuntu
```

Then follow the [Linux](#linux) section inside the WSL2 shell. Two caveats:

- **Clone inside the WSL filesystem** (e.g. `~/Jaster`), not under `/mnt/c/...`.
  Cargo builds across the filesystem boundary are dramatically slower and can
  hit file-locking issues.
- **WSL2 does not pass through host HID devices**, so `jaster event` finds
  nothing and `jaster start` exits with "No keyboards found". Use WSL2 for
  compiling and reviewing, not for hardware verification.

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
│   ├── start.rs          spawns the detached daemon, writes the PID file
│   ├── daemon.rs         the real work: audio engine + sound cache + the key listener
│   ├── stop.rs           reads the PID file, terminates the daemon
│   ├── doctor.rs         environment diagnostics, per platform
│   ├── sounds.rs         lists the installed sound packs
│   ├── switch.rs         `jaster oreo` — clap external subcommand; restarts the daemon
│   ├── volume.rs         `jaster volume` — shows/sets the saved level
│   ├── event.rs          lists detected keyboards
│   ├── update.rs         self-update via the GitHub release installer
│   └── version.rs        prints CARGO_PKG_VERSION
├── keyboard/             the only place a key-capture backend lives
│   ├── key.rs            `Key` — a PS/2 set-1 scancode, the type the rest of the code uses
│   ├── mod.rs            picks the backend by target; all three must export sources() + listen()
│   ├── linux.rs          scans /dev/input for A + ENTER + SPACE, one thread per device
│   ├── macos.rs          CGEventTap, its run loop, and the Input Monitoring checks
│   ├── macos_keys.rs     the testable half of the macOS backend: keycodes + modifier state
│   └── windows.rs        WH_KEYBOARD_LL hook, its message pump, and the auto-repeat filter
├── audio/
│   ├── engine.rs         rodio output stream
│   ├── theme.rs          discovers sound packs and parses their config.json
│   ├── cache.rs          decodes a pack into one in-memory clip per key, levelled
│   ├── volume.rs         the saved level, plus the daemon's live view of it
│   └── player.rs         plays a buffered sound into the mixer
├── utils/select.rs       raw-mode arrow-key picker used by `jaster start`
├── utils/paths.rs        where the data dir and sound packs live, per platform
└── utils/pid.rs          PID file helpers, including the "is it still Jaster?" check
```

### Where the platform differences are

Only two areas branch on the OS, and keeping it that way is the point:

- **`src/keyboard/`** — the backend is selected in `mod.rs` with
  `#[cfg_attr(..., path = ...)]`, so `linux.rs`, `macos.rs` and `windows.rs` are
  never compiled together and must all satisfy the same two-function interface.
  `macos_keys.rs` is the exception that proves it: it holds the parts of the
  macOS backend that need no framework, so they can be compiled and tested on
  every host.
- **`#[cfg(unix)]` / `#[cfg(windows)]` arms** in `utils/paths.rs`,
  `utils/pid.rs`, `utils/select.rs`, `commands/start.rs`, `commands/stop.rs`,
  `commands/doctor.rs`, and `commands/update.rs` — each is a small set of
  functions with the same signature, sitting next to each other. Where macOS
  parts company with Linux inside `unix` it gets its own arm: `installed_sounds`
  in `paths.rs` (`/usr/local/share`, because SIP seals `/usr/share`) and
  `is_jaster` in `pid.rs` (`proc_pidpath`, because there is no `/proc`).

`src/audio/` contains no platform code at all, which is why `Key` exists.

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

## Contribution workflow

1. Fork the repository and create a branch:

   ```bash
   git checkout -b feat/short-description
   ```

2. Make your change. Before committing:

   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo build
   cargo test
   ```

   If you touched anything `#[cfg]`-gated, type-check the two targets your
   machine is not:

   ```bash
   cargo clippy --target aarch64-apple-darwin --all-targets -- -D warnings
   cargo clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings
   ```

3. Write a clear commit message describing what changed and why.

4. Open a pull request against `main`. In the description, state:
   - which OS you developed and tested on;
   - whether you were able to verify runtime behavior on real hardware, or only
     that it compiles (entirely fine — just say so, so a maintainer knows to
     smoke-test it).

CI runs clippy and the test suite on Linux, macOS and Windows for every push and
pull request, plus a cross-check of both Apple targets from Linux. It cannot
type for you, though: nothing in CI presses a key, so hardware behavior is still
something a human has to confirm.

### Version bumps

The release version lives in `Cargo.toml` and must be committed together with
the updated `Cargo.lock` — several past commits exist purely to repair a
mismatch between the two. Everything else reads it from there: `jaster version`
through `env!("CARGO_PKG_VERSION")`, and `jaster --version` through clap's bare
`#[command(version)]` in `src/cli/args.rs`.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Jaster supports Linux, macOS and Windows` compile error | Building on a fourth OS | Expected — there is no backend for it in `src/keyboard/` |
| `ALSA lib ... cannot find card` or link error on `-lasound` | Missing ALSA headers | Install `libasound2-dev` / `alsa-lib-devel` |
| `jaster doctor` reports "Permission denied" | User is not in the `input` group | `sudo usermod -aG input $USER`, then start a new session |
| No keyboards detected, permissions look correct | Session not refreshed since the group change | `exec su - "$USER"`, or log out and back in |
| No keyboards detected inside WSL2 | WSL2 does not expose host HID devices | Expected — verify on real Linux hardware or a VM |
| Daemon runs but no sound | Wrong working directory, so a dev build loaded installed assets | Run `cargo run` from the repository root |
| `jaster stop` says "stopped" but sound continues | Stale PID file | `pkill -f 'jaster daemon'` and remove `~/.local/share/jaster/jaster.pid` |
| Windows: `jaster start` succeeds but nothing plays | Daemon failed after detaching | Read `%LOCALAPPDATA%\Jaster\daemon.log` |
| Windows: typing is silent only in some apps | The app runs elevated, or anti-cheat blocks the hook | Run `jaster doctor`; elevate Jaster to match |
| Windows: a held key machine-guns | Auto-repeat filter regressed in `src/keyboard/windows.rs` | The `HELD` set must be updated on key-up |
| macOS: no sound at all, `doctor` shows the tap unavailable | Input Monitoring not granted to the terminal you launched from | Grant it, then quit the terminal with ⌘Q and reopen it |
| macOS: granted the permission and still nothing | The grant only reaches processes started after it, and it names your terminal, not Jaster | ⌘Q the terminal — a new window is not enough — and check the entry is the terminal's |
| macOS: silent in one app only | That app uses Secure Keyboard Entry, which excludes every event tap | Expected; nothing Jaster can do |
| macOS: sound stops after a burst of typing, daemon still running | The tap was disabled by timeout or user input and not re-armed | `handle` in `src/keyboard/macos.rs` must call `CGEventTapEnable` on both disable events |
| macOS: one key plays the generic click | Its virtual keycode is missing from the table | Add it to `from_virtual` in `src/keyboard/macos_keys.rs`; the tests there cover the rest |
| macOS: `jaster stop` says "stopped" but sound continues | `is_jaster` failed to match the running binary | Check the `proc_pidpath` arm in `src/utils/pid.rs`; `pkill jaster` meanwhile |

---

## Getting help

- Open an issue: <https://github.com/JoeCelaster/Jaster/issues>
- Run `cargo run -- doctor` and paste the output — it is the fastest way for a
  maintainer to see your environment.
