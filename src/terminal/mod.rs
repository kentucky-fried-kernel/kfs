mod cursor;
pub mod ps2;
#[allow(clippy::module_inception)]
mod terminal;
mod vga;

pub use terminal::Terminal;
pub use vga::Color;
