#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]

pub mod cursor;
pub mod entry;
pub mod screen;
#[allow(clippy::module_inception)]
pub mod vga;

pub use screen::*;
