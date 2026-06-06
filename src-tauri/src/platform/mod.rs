#[cfg(target_os = "macos")]
mod network_macos;

#[cfg(target_os = "macos")]
pub use network_macos::*;

#[cfg(target_os = "windows")]
mod network_windows;

#[cfg(target_os = "windows")]
pub use network_windows::*;
