#![allow(unused)]
#![allow(static_mut_refs)]
#![allow(clippy::cast_possible_truncation)]

use crate::{printk, printkln, serial_println};

const MAX_INTERRUPT_DESCRIPTORS: usize = 256;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct InterruptDescriptor {
    isr_low: u16,
    kernel_cs: u16,
    zero: u8,
    attributes: u8,
    isr_high: u16,
}

#[repr(C, packed)]
struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: usize,
}

#[repr(C, align(0x10))]
struct InterruptDescriptorTable {
    pub entries: [InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS],
    pub idtr: InterruptDescriptorTableRegister,
}

#[repr(u8)]
enum GateType {
    TaskGate = 0b0101,
    InterruptGate16 = 0b0110,
    TrapGate16 = 0b0111,
    InterruptGate32 = 0b1110,
    TrapGate32 = 0b1111,
}

fn build_attributes(present: u8, privilege_level: u8, gate_type: GateType) -> u8 {
    (present << 7) | (privilege_level << 5) | gate_type as u8
}

impl InterruptDescriptor {
    pub fn new(offset: usize, kernel_cs: u16, attributes: u8) -> Self {
        Self {
            isr_low: (offset & 0xFFFF) as u16,
            kernel_cs,
            zero: 0,
            attributes,
            isr_high: ((offset >> 16) & 0xFFFF) as u16,
        }
    }

    pub fn offset(&self) -> u32 {
        (u32::from(self.isr_high) << 16) | u32::from(self.isr_low)
    }
}

impl InterruptDescriptorTable {
    /// Creates a new `InterruptDescriptorTable` filled with non-present
    /// entries.
    pub fn new() -> Self {
        let mut idt = Self {
            entries: [InterruptDescriptor::new(0, 0, 0); MAX_INTERRUPT_DESCRIPTORS],
            idtr: InterruptDescriptorTableRegister { base: 0, limit: 0 },
        };

        idt.idtr.base = idt.entries.as_ptr() as usize;
        idt.idtr.limit = (core::mem::size_of::<[InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS]>() - 1) as u16;

        idt
    }

    pub fn load(&self) {
        let idtr = InterruptDescriptorTableRegister {
            base: self.entries.as_ptr() as usize,
            limit: (core::mem::size_of::<[InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS]>() - 1) as u16,
        };

        // SAFETY:
        // We are using inline assembly to get access to the `lidt` instruction. The
        // value we pass to it contains the address to a static IDT, which is
        // guaranteed stay valid for the entire lifetime of the program.
        unsafe {
            core::arch::asm!("lidt [{}]", "sti", in(reg) &raw const idtr, options(readonly, nostack, preserves_flags));
        }
    }

    pub fn set_descriptor(&mut self, index: u8, descriptor: InterruptDescriptor) {
        self.entries[index as usize] = descriptor;
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn exception_handler() {
    core::arch::naked_asm!("pusha", "call handle_exception", "popa", "iret")
}

#[unsafe(no_mangle)]
extern "C" fn handle_exception() {
    // panic!("FUCK");
}

macro_rules! isr_no_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!(
                "pusha",
                "push {}",
                "call handle_interrupt",
                "add esp, 4",
                "popa",
                "iretd",
                const $nb
            )
        }
    };
}

macro_rules! isr_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!(
                "add esp, 4",
                "pusha",
                "push {}",
                "call handle_interrupt",
                "add esp, 4",
                "popa",
                "iretd",
                const $nb
            )
        }
    };
}

#[unsafe(no_mangle)]
extern "C" fn handle_interrupt(interrupt_number: usize) {
    serial_println!("INT {} received", interrupt_number);

    match interrupt_number {
        13 => panic!("General Protection Fault"),
        14 => panic!("Page Fault"),
        _ => {
            serial_println!("Handling interrupt {}, about to return", interrupt_number);

            if interrupt_number >= 32 && interrupt_number < 48 {
                remap_pic();
                unsafe {
                    use crate::port::Port;

                    if interrupt_number >= 40 {
                        Port::new(0xA0).write(0x20u8);
                    }
                    Port::new(0x20).write(0x20u8);
                }
            }
        }
    }
}

isr_no_err_stub!(isr_stub_0, 0);
isr_no_err_stub!(isr_stub_1, 1);
isr_no_err_stub!(isr_stub_2, 2);
isr_no_err_stub!(isr_stub_3, 3);
isr_no_err_stub!(isr_stub_4, 4);
isr_no_err_stub!(isr_stub_5, 5);
isr_no_err_stub!(isr_stub_6, 6);
isr_no_err_stub!(isr_stub_7, 7);
isr_err_stub!(isr_stub_8, 8);
isr_no_err_stub!(isr_stub_9, 9);
isr_err_stub!(isr_stub_10, 10);
isr_err_stub!(isr_stub_11, 11);
isr_err_stub!(isr_stub_12, 12);
isr_err_stub!(isr_stub_13, 13);
isr_err_stub!(isr_stub_14, 14);
isr_no_err_stub!(isr_stub_15, 15);
isr_no_err_stub!(isr_stub_16, 16);
isr_err_stub!(isr_stub_17, 17);
isr_no_err_stub!(isr_stub_18, 18);
isr_no_err_stub!(isr_stub_19, 19);
isr_no_err_stub!(isr_stub_20, 20);
isr_no_err_stub!(isr_stub_21, 21);
isr_no_err_stub!(isr_stub_22, 22);
isr_no_err_stub!(isr_stub_23, 23);
isr_no_err_stub!(isr_stub_24, 24);
isr_no_err_stub!(isr_stub_25, 25);
isr_no_err_stub!(isr_stub_26, 26);
isr_no_err_stub!(isr_stub_27, 27);
isr_no_err_stub!(isr_stub_28, 28);
isr_no_err_stub!(isr_stub_29, 29);
isr_err_stub!(isr_stub_30, 30);
isr_no_err_stub!(isr_stub_31, 31);
isr_no_err_stub!(isr_stub_32, 32);
isr_no_err_stub!(isr_stub_33, 33);
isr_no_err_stub!(isr_stub_34, 34);
isr_no_err_stub!(isr_stub_35, 35);
isr_no_err_stub!(isr_stub_36, 36);
isr_no_err_stub!(isr_stub_37, 37);
isr_no_err_stub!(isr_stub_38, 38);
isr_no_err_stub!(isr_stub_39, 39);
isr_no_err_stub!(isr_stub_40, 40);
isr_no_err_stub!(isr_stub_41, 41);
isr_no_err_stub!(isr_stub_42, 42);
isr_no_err_stub!(isr_stub_43, 43);
isr_no_err_stub!(isr_stub_44, 44);
isr_no_err_stub!(isr_stub_45, 45);
isr_no_err_stub!(isr_stub_46, 46);
isr_no_err_stub!(isr_stub_47, 47);

macro_rules! isr_stubs {
    () => {
        &[
            isr_stub_0 as *const () as usize,
            isr_stub_1 as *const () as usize,
            isr_stub_2 as *const () as usize,
            isr_stub_3 as *const () as usize,
            isr_stub_4 as *const () as usize,
            isr_stub_5 as *const () as usize,
            isr_stub_6 as *const () as usize,
            isr_stub_7 as *const () as usize,
            isr_stub_8 as *const () as usize,
            isr_stub_9 as *const () as usize,
            isr_stub_10 as *const () as usize,
            isr_stub_11 as *const () as usize,
            isr_stub_12 as *const () as usize,
            isr_stub_13 as *const () as usize,
            isr_stub_14 as *const () as usize,
            isr_stub_15 as *const () as usize,
            isr_stub_16 as *const () as usize,
            isr_stub_17 as *const () as usize,
            isr_stub_18 as *const () as usize,
            isr_stub_19 as *const () as usize,
            isr_stub_20 as *const () as usize,
            isr_stub_21 as *const () as usize,
            isr_stub_22 as *const () as usize,
            isr_stub_23 as *const () as usize,
            isr_stub_24 as *const () as usize,
            isr_stub_25 as *const () as usize,
            isr_stub_26 as *const () as usize,
            isr_stub_27 as *const () as usize,
            isr_stub_28 as *const () as usize,
            isr_stub_29 as *const () as usize,
            isr_stub_30 as *const () as usize,
            isr_stub_31 as *const () as usize,
            isr_stub_32 as *const () as usize,
            isr_stub_33 as *const () as usize,
            isr_stub_34 as *const () as usize,
            isr_stub_35 as *const () as usize,
            isr_stub_36 as *const () as usize,
            isr_stub_37 as *const () as usize,
            isr_stub_38 as *const () as usize,
            isr_stub_39 as *const () as usize,
            isr_stub_40 as *const () as usize,
            isr_stub_41 as *const () as usize,
            isr_stub_42 as *const () as usize,
            isr_stub_43 as *const () as usize,
            isr_stub_44 as *const () as usize,
            isr_stub_45 as *const () as usize,
            isr_stub_46 as *const () as usize,
            isr_stub_47 as *const () as usize,
        ]
    };
}

pub fn remap_pic() {
    const PIC1_CMD: u16 = 0x20;
    const PIC1_DATA: u16 = 0x21;
    const PIC2_CMD: u16 = 0xA0;
    const PIC2_DATA: u16 = 0xA1;

    const ICW1_INIT: u8 = 0x10;
    const ICW1_ICW4: u8 = 0x01;
    const ICW4_8086: u8 = 0x01;

    // SAFETY:
    // We are using the `Port` struct to write to the programmable interrupt
    // controller.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        use crate::port::Port;

        _ = Port::new(PIC1_DATA).read();
        _ = Port::new(PIC2_DATA).read();

        Port::new(PIC1_CMD).write(ICW1_INIT | ICW1_ICW4);
        Port::new(PIC2_CMD).write(ICW1_INIT | ICW1_ICW4);

        Port::new(PIC1_DATA).write(0x20);
        Port::new(PIC2_DATA).write(0x28);

        Port::new(PIC1_DATA).write(4);
        Port::new(PIC2_DATA).write(2);

        Port::new(PIC1_DATA).write(ICW4_8086);
        Port::new(PIC2_DATA).write(ICW4_8086);

        Port::new(PIC1_DATA).write(0xFF);
        Port::new(PIC2_DATA).write(0xFF);
    }
}

static mut IDT: Option<InterruptDescriptorTable> = None;

pub fn init() {
    let stubs = isr_stubs!();
    let mut idt = InterruptDescriptorTable::new();

    for (index, stub) in stubs.iter().enumerate() {
        idt.set_descriptor(
            index as u8,
            InterruptDescriptor::new(*stub, 0x08, build_attributes(1, 0, GateType::InterruptGate32)),
        );
    }

    unsafe {
        IDT = Some(idt);

        remap_pic();

        if let Some(ref idt) = IDT {
            idt.load();
        }
    }
}
