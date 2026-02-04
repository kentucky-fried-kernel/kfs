use crate::{
    arch::x86::idt::InterruptRegisters,
    port::Port,
    ps2::{
        DATA_PORT, Key,
        scancodes::{KeyEvent, SCANCODE_SET_1_TO_KEY, SCANCODE_SET_1_TO_KEY_EXTENDED},
    },
    serial_println,
};

const BUFFER_SIZE: usize = 256;

static mut BUFFER: Buffer = Buffer::new();
static mut EXTENDED_BYTE_SENT: bool = false;

struct Buffer {
    buffer: [Option<KeyEvent>; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl Buffer {
    const fn new() -> Self {
        Self {
            buffer: [None; BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    fn push(&mut self, key_event: KeyEvent) {
        let next_write = (self.write_pos + 1) % BUFFER_SIZE;

        if next_write == self.read_pos {
            self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;
        }
        self.buffer[self.write_pos] = Some(key_event);
        self.write_pos = next_write;
    }

    fn pop(&mut self) -> Option<KeyEvent> {
        if self.read_pos == self.write_pos {
            return None;
        }
        let key = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;

        key
    }
}

/// IRQ1
extern "C" fn keyboard_interrupt_handler(_regs: &InterruptRegisters) {
    let data_port = Port::new(DATA_PORT);
    let scancode = unsafe { data_port.read() };

    // SAFETY
    // This is a global variable which will be available throughout
    // the whole runtime of the program
    let key = if unsafe { EXTENDED_BYTE_SENT } {
        unsafe {
            EXTENDED_BYTE_SENT = false;
        }
        SCANCODE_SET_1_TO_KEY_EXTENDED[scancode as usize]
    } else {
        if scancode == 0xE0 {
            unsafe {
                EXTENDED_BYTE_SENT = true;
            }
            None
        } else {
            SCANCODE_SET_1_TO_KEY[scancode as usize]
        }
    };

    if let Some(key) = key {
        unsafe {
            serial_println!("{:?}", key);
            #[allow(static_mut_refs)]
            BUFFER.push(key);
        }
    }
}

#[must_use]
pub fn read_key_event() -> Option<KeyEvent> {
    #[allow(static_mut_refs)]
    unsafe {
        BUFFER.pop()
    }
}

pub fn init() {
    use crate::arch::x86::interrupts::irq;

    irq::install_handler(1, keyboard_interrupt_handler);

    irq::clear_mask(1);
}
