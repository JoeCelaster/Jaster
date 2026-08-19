mod key;

pub use key::Key;

#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod backend;

/// The macOS keycode table and its modifier logic live outside the backend so
/// `cargo test` compiles and runs them on Linux and Windows too. They are pure
/// arithmetic — the only part of the macOS port that can be verified without a
/// Mac, so that is where the verification effort goes.
#[cfg(any(target_os = "macos", test))]
mod macos_keys;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
compile_error!("Jaster supports Linux, macOS and Windows.");

pub use backend::{listen, sources};

#[cfg(windows)]
pub use backend::hook_is_available;

#[cfg(target_os = "macos")]
pub use backend::{Access, access, request, tap_can_be_created};
