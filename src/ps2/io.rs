use crate::port::Port;

use super::{
    COMMAND_PORT, DATA_PORT, Key, OUTPUT_BUFFER_STATUS_BIT, STATUS_PORT,
    controller::{Command, Status},
    scancodes::SCANCODE_TO_KEY,
};

pub fn send_command(cmd: Command) {
    let status_port = Port::new(STATUS_PORT);
    while unsafe { status_port.read() } & Status::InputFull as u8 != 0 {}

    unsafe { Port::new(COMMAND_PORT).write(cmd as u8) };
}

pub fn send_data(data: u8) {
    let status_port = Port::new(STATUS_PORT);
    while unsafe { status_port.read() } & Status::InputFull as u8 != 0 {}

    unsafe { Port::new(DATA_PORT).write(data) };
}

pub fn wait_for_data() -> u8 {
    let status_port = Port::new(STATUS_PORT);
    while unsafe { status_port.read() } & Status::OutputFull as u8 == 0 {}

    unsafe { Port::new(DATA_PORT).read() }
}

/// Reads all data from the output buffer, flushing it. Note that this will
/// go into an endless loop if called without disabling the ports first.
pub fn flush_output_buffer() {
    let status_port = Port::new(STATUS_PORT);
    while unsafe { status_port.read() } & Status::OutputFull as u8 != 0 {
        unsafe { Port::new(DATA_PORT).read() };
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
    let data_port = Port::new(DATA_PORT);
    let code = unsafe { data_port.read() };

    if code == 0xF0 || code == 0xE0 {
        while !is_ps2_data_available() {}
        let _ = unsafe { data_port.read() };
        unsafe { LAST_KEY = None };
        return None;
    }

    unsafe { LAST_KEY = Some(code) };
    SCANCODE_TO_KEY[code as usize].1
}

/// Returns `true` if the PS2 input buffer has data ready to be read,
/// meaning the least significant bit of the PS2 status port is set.
fn is_ps2_data_available() -> bool {
    let status_port = Port::new(STATUS_PORT);
    unsafe { status_port.read() & OUTPUT_BUFFER_STATUS_BIT != 0 }
}
