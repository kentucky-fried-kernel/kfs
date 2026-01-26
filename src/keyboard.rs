//! Keyboard driver with interrupt-based input

#![allow(static_mut_refs)]

use crate::{
    arch::x86::idt::InterruptRegisters,
    port::Port,
    ps2::{
        DATA_PORT, Key,
        scancodes::{SCANCODE_TO_KEY, SCANCODE_TO_KEY_SECOND},
    },
};

const BUFFER_SIZE: usize = 256;

static mut KEYBOARD_BUFFER: KeyboardBuffer = KeyboardBuffer::new();
static mut EXTENDED_BYTE_SENT: bool = false;

struct KeyboardBuffer {
    buffer: [Option<Key>; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            buffer: [None; BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    fn push(&mut self, key: Key) {
        let next_write = (self.write_pos + 1) % BUFFER_SIZE;

        if next_write == self.read_pos {
            self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;
        }
        self.buffer[self.write_pos] = Some(key);
        self.write_pos = next_write;
    }

    fn pop(&mut self) -> Option<Key> {
        if self.read_pos == self.write_pos {
            return None;
        }
        let key = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;

        key
    }
}

/// IRQ1
pub extern "C" fn keyboard_interrupt_handler(_regs: &InterruptRegisters) {
    let data_port = Port::new(DATA_PORT);
    let scancode = unsafe { data_port.read() };

    let key = if unsafe { EXTENDED_BYTE_SENT } == true {
        unsafe {
            EXTENDED_BYTE_SENT = false;
        }
        SCANCODE_TO_KEY_SECOND[scancode as usize].1
    } else {
        if scancode == 0xE0 {
            unsafe {
                EXTENDED_BYTE_SENT = true;
            }
            None
        } else {
            SCANCODE_TO_KEY[scancode as usize].1
        }
    };

    if let Some(key) = key {
        unsafe {
            KEYBOARD_BUFFER.push(key);
        }
    }
}

#[must_use]
pub fn read_key() -> Option<Key> {
    unsafe { KEYBOARD_BUFFER.pop() }
}

pub fn init() {
    use crate::arch::x86::interrupts::irq;

    irq::install_handler(1, keyboard_interrupt_handler);

    irq::clear_mask(1);
}
