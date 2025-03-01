mod controller;
mod scancodes;

pub use controller::{init, read_if_ready};
pub use scancodes::Key;
