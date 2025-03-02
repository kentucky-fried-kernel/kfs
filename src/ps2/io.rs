use core::arch::asm;

use super::{
    COMMAND_PORT, DATA_PORT, Key, OUTPUT_BUFFER_STATUS_BIT, STATUS_PORT,
    controller::{Command, Status},
    scancodes::SCANCODE_TO_KEY,
};

pub fn send_command(cmd: Command) {
    while unsafe { read(STATUS_PORT) } & Status::InputFull as u8 != 0 {}

    unsafe { write(COMMAND_PORT, cmd as u8) };
}

pub fn send_data(data: u8) {
    while unsafe { read(STATUS_PORT) } & Status::InputFull as u8 != 0 {}

    unsafe { write(DATA_PORT, data) };
}

pub fn wait_for_data() -> u8 {
    while unsafe { read(STATUS_PORT) } & Status::OutputFull as u8 == 0 {}

    unsafe { read(DATA_PORT) }
}

/// Reads all data from the output buffer, flushing it. Note that this will
/// go into an endless loop if called without disabling the ports first.
pub fn flush_output_buffer() {
    while unsafe { read(STATUS_PORT) } & Status::OutputFull as u8 != 0 {
        unsafe { read(DATA_PORT) };
    }
}

static mut LAST_KEY: Option<u8> = None;

/// Reads from the PS2 data port if the PS2 status port is ready. Returns `Some(KeyScanCode)`
/// if the converted scancode is a supported character.
///
/// /// ### Example Usage:
/// ```
/// let mut v = Vga::new();
///
/// if let Some(c) = read_if_ready() == KeyScanCode::A {
///     v.write_char(b'a');
/// }
pub fn read_if_ready() -> Option<Key> {
    if !is_ps2_data_available() {
        return None;
    }

    let code = unsafe { read(DATA_PORT) };

    if code == 0xF0 || code == 0xE0 {
        while !is_ps2_data_available() {}
        let _ = unsafe { read(DATA_PORT) };
        unsafe { LAST_KEY = None };
        return None;
    }

    unsafe { LAST_KEY = Some(code) };
    SCANCODE_TO_KEY[code as usize].1
}

/// Returns `true` if the PS2 input buffer has data ready to be read,
/// meaning the least significant bit of the PS2 status port is set.
fn is_ps2_data_available() -> bool {
    status() & OUTPUT_BUFFER_STATUS_BIT != 0
}

/// Reads from `STATUS_PORT` and returns the extracted value.
fn status() -> u8 {
    let res: u8;

    unsafe {
        res = read(STATUS_PORT);
    }

    res
}

/// Reads from `port` and returns the extracted value.
pub unsafe fn read(port: u16) -> u8 {
    assert!(port == DATA_PORT || port == STATUS_PORT);

    let res: u8;

    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") res,
        );
    }

    res
}

unsafe fn write(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
        );
    }
}
