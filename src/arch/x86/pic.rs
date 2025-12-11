//! https://wiki.osdev.org/8259_PIC

use crate::port::Port;

/// I/O base address for master PIC
const PIC1: u8 = 0x20;
const PIC1_COMMAND: u8 = PIC1;
const PIC1_DATA: u8 = PIC1 + 1;

/// I/O base address for slave PIC
const PIC2: u8 = 0xA0;
const PIC2_COMMAND: u8 = PIC2;
const PIC2_DATA: u8 = PIC2 + 1;

/// End-of-interrupt command code
const PIC_EOI: u8 = 0x20;

/// Sends an end-of-interrupt command code to the PIC responsible for `IRQ`.
pub fn send_eoi(irq: u8) {
    if irq >= 8 {
        // SAFETY:
        // We are writing to the PIC1 port, which we assume to be safe.
        unsafe { Port::new(PIC2_COMMAND as u16).write(PIC_EOI) };
    }
    // SAFETY:
    // We are writing to the PIC1 port, which we assume to be safe.
    unsafe { Port::new(PIC1_COMMAND as u16).write(PIC_EOI) };
}

#[bitstruct::bitstruct]
struct ICW1 {
    /// Initialization
    init: u1,

    /// Level triggered (edge) mode
    level: u1,

    /// Call address interval
    interval4: u1,

    /// Single (cascade) mode
    single: u1,

    /// ICW4 will be present
    icw4: u1,
}

#[bitstruct::bitstruct]
struct ICW4 {
    /// Special fully nested (not)
    sfnm: u1,

    /// Buffered mode for master
    buf_master: u1,

    /// Buffered mode for slave
    buf_slave: u1,

    /// Auto (normal) EOI
    auto: u1,

    /// 8086/88 mode
    mode_8086: u1,
}

const CASCADE_IRQ: u8 = 2;

pub fn remap(offset1: u32, offset2: u32) {
    let mut pic1_command = Port::new(PIC1_COMMAND as u16);
    let mut pic1_data = Port::new(PIC1_DATA as u16);
    let mut pic2_command = Port::new(PIC2_COMMAND as u16);
    let mut pic2_data = Port::new(PIC2_DATA as u16);

    // SAFETY:
    // We are writing to the PIC1/PIC2 ports, which we assume to be safe.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        pic1_command.write(ICW1::new(0).set_init(1).set_icw4(1).0);
        pic2_command.write(ICW1::new(0).set_init(1).set_icw4(1).0);
        pic1_data.write(offset1 as u8);
        pic2_data.write(offset2 as u8);
        pic1_data.write(1 << CASCADE_IRQ);
        pic2_data.write(2);
        pic1_data.write(ICW4::new(0).set_mode_8086(1).0);
        pic2_data.write(ICW4::new(0).set_mode_8086(1).0);
        pic1_data.write(0);
        pic2_data.write(0);
    };
}

pub fn disable() {
    // SAFETY:
    // We are writing to the PIC1/PIC2 ports, which we assume to be safe.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        Port::new(PIC1_DATA as u16).write(0xff);
        Port::new(PIC2_DATA as u16).write(0xff);
    };
}

mod irq {
    use crate::{
        arch::x86::pic::{PIC1_DATA, PIC2_DATA},
        port::Port,
    };

    pub fn set_mask(mut irq_line: u8) {
        let mut port = Port::new(if let 0..8 = irq_line {
            PIC1_DATA
        } else {
            irq_line -= 8;
            PIC2_DATA
        } as u16);

        // SAFETY:
        // We are writing to the PIC1/PIC2 ports, which we assume to be safe.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            let val = port.read() | (1 << irq_line);
            port.write(val);
        }
    }

    pub fn clear_mask(mut irq_line: u8) {
        let mut port = Port::new(if let 0..8 = irq_line {
            PIC1_DATA
        } else {
            irq_line -= 8;
            PIC2_DATA
        } as u16);

        // SAFETY:
        // We are writing to the PIC1/PIC2 ports, which we assume to be safe.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            let val = port.read() & !(1 << irq_line);
            port.write(val);
        }
    }
}
