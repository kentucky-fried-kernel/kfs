mod controller;
mod io;
pub mod scancodes;

pub const DATA_PORT: u16 = 0x60;
pub const STATUS_PORT: u16 = 0x64;
pub const COMMAND_PORT: u16 = 0x64;
pub const OUTPUT_BUFFER_STATUS_BIT: u8 = 1;

pub use scancodes::Key;
