mod key;

pub use key::Key;

#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod backend;

#[cfg(not(any(target_os = "linux", windows)))]
compile_error!("Jaster supports Linux and Windows. See SETUP.md for the macOS port.");

pub use backend::{listen, sources};

#[cfg(windows)]
pub use backend::hook_is_available;
