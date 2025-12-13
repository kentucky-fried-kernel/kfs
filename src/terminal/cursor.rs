use core::arch::asm;

use crate::terminal::vga::{BUFFER_HEIGHT, BUFFER_WIDTH};

/// Abstraction for managing the [Text-mode cursor](https://wiki.osdev.org/Text_Mode_Cursor).
#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub x: u16,
    pub y: u16,
}

impl Cursor {
    const LOCATION_REG_LOW: u8 = 0x0F;
    const LOCATION_REG_HIGH: u8 = 0x0E;
    const REG_START: u8 = 0x0A;
    const REG_END: u8 = 0x0B;

    #[must_use]
    pub fn new(x: u16, y: u16) -> Self {
        Cursor { x, y }
    }

    /// Flushes the text-mode cursor position in the VGA buffer by setting the
    /// CRTC's [location registers](http://www.osdever.net/FreeVGA/vga/crtcreg.htm#0F) (`0x0F` and `0x0D`)
    /// to `x, y`.
    ///
    /// ## SAFETY
    /// 1. This function uses `Cursor::update`, which writes directly to the VGA buffer. In
    ///    user-mode, this **will** result in invalid memory access.
    ///
    /// 2. `flush_pos` may cause undefined behavior if called with `x` or `y` values outside of the
    ///    range `0x00..=0x0F`.
    pub unsafe fn flush_pos(&self) {
        let out_of_bounds: bool = !(0..BUFFER_HEIGHT).contains(&(self.y as usize)) || !(0..BUFFER_WIDTH).contains(&(self.x as usize));
        if out_of_bounds {
            return;
        }

        let pos = self.y * BUFFER_WIDTH as u16 + self.x;

        unsafe {
            Self::update(Cursor::LOCATION_REG_LOW, (pos & 0xFF) as u8);
            Self::update(Cursor::LOCATION_REG_HIGH, ((pos >> 8) & 0xFF) as u8);
        }
    }

    /// Resizes the cursor by updating the [cursor end & start register](http://www.osdever.net/FreeVGA/vga/crtcreg.htm#0A)
    /// (`0x0A` and `0x0B`) to `start, end`. The values of `start` and `end` are
    /// expected to be in the range `0x00..=0x0F`.
    ///
    /// ## SAFETY
    /// 1. This function uses `Cursor::update`, which writes directly to the VGA buffer. In
    ///    user-mode, this **will** result in invalid memory access.
    ///
    /// 2. `resize` may cause undefined behavior if called with `start` or `end` values outside of
    ///    the range `0x00..=0x0F`.
    pub unsafe fn resize(start: u8, end: u8) {
        unsafe {
            Self::update(Cursor::REG_START, start);
            Self::update(Cursor::REG_END, end);
        }
    }

    /// Abstraction for the ugliness behind updating the cursor.
    ///
    /// `0x3D4` is the I/O port address for the VGA's CRTC ([Cathode-ray tube](https://en.wikipedia.org/wiki/Cathode-ray_tube))'s
    /// index register. The value being loaded into it defines which CRTC
    /// functionality we want to access. The different indices that can be loaded into it are documented [here](http://www.osdever.net/FreeVGA/vga/crtcreg.htm#0A).
    ///
    /// After the index has been loaded into the `0x3D4`, `dx`, (where the index
    /// register is stored) can be incremented by one. This will move it to
    /// `0x3D5`, the CRTC's data register, signifying the CRTC's readiness to
    /// receive the input values.
    ///
    /// ## SAFETY:
    /// This writes to the VGA buffer directly, running this in a non-bare-metal
    /// environment will result in invalid memory access.
    unsafe fn update(index: u8, value: u8) {
        unsafe {
            asm!(
                "mov dx, 0x3D4",
                "mov al, {index}",
                "out dx, al",
                "inc dx",
                "mov al, {value}",
                "out dx, al",
                index = in(reg_byte) (index),
                value = in(reg_byte) (value),
                out("dx") _,
                out("al") _,
            );
        }
    }

    pub fn show() {
        unsafe {
            Self::resize(0, 15);
        }
    }

    pub fn hide() {
        unsafe {
            Self::update(Cursor::REG_START, 1 << 5);
        }
    }
}
