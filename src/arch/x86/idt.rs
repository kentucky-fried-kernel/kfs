#![allow(static_mut_refs)]
#![allow(unused)]

use crate::port::Port;

const MAX_INTERRUPT_DESCRIPTORS: usize = 256;

#[repr(C)]
struct InterruptStackFrame {
    ip: u32,
    cs: u32,
    flags: u32,
    sp: u32,
    ss: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct InterruptDescriptor {
    isr_low: u16,
    selector: u16,
    zero: u8,
    type_attributes: u8,
    isr_high: u16,
}

#[repr(C, packed)]
struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: usize,
}

#[repr(C, align(16))]
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

struct TypeAttribute(u8);

fn build_type_attributes(present: u8, privilege_level: u8, gate_type: GateType) -> u8 {
    (present << 7) | (privilege_level << 5) | gate_type as u8
}

impl InterruptDescriptor {
    pub fn new(offset: usize, selector: u16, type_attributes: u8) -> Self {
        Self {
            isr_low: (offset & 0xFFFF) as u16,
            selector,
            zero: 0,
            type_attributes,
            isr_high: ((offset >> 16) & 0xFFFF) as u16,
        }
    }

    pub fn offset(&self) -> u32 {
        ((self.isr_high as u32) << 16) | self.isr_low as u32
    }
}

impl InterruptDescriptorTable {
    /// Creates a new `InterruptDescriptorTable` filled with non-present entries.
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
        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) &self.idtr, options(readonly, nostack, preserves_flags));
            core::arch::asm!("sti")
        }
    }

    pub fn set_descriptor(&mut self, index: u8, descriptor: InterruptDescriptor) {
        self.entries[index as usize] = descriptor;
    }
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn exception_handler() {
    unsafe { core::arch::naked_asm!("cli; hlt") };
}

macro_rules! isr_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            // panic!("???");
            core::arch::naked_asm!("call exception_handler", "iret")
        }
    };
}

macro_rules! isr_no_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            // panic!("???");
            core::arch::naked_asm!("call exception_handler", "iret")
        }
    };
}

isr_no_err_stub!(isr_no_err_stub_0, 0);
isr_no_err_stub!(isr_no_err_stub_1, 1);
isr_no_err_stub!(isr_no_err_stub_2, 2);
isr_no_err_stub!(isr_no_err_stub_3, 3);
isr_no_err_stub!(isr_no_err_stub_4, 4);
isr_no_err_stub!(isr_no_err_stub_5, 5);
isr_no_err_stub!(isr_no_err_stub_6, 6);
isr_no_err_stub!(isr_no_err_stub_7, 7);
isr_err_stub!(isr_err_stub_8, 8);
isr_no_err_stub!(isr_no_err_stub_9, 9);
isr_err_stub!(isr_err_stub_10, 10);
isr_err_stub!(isr_err_stub_11, 11);
isr_err_stub!(isr_err_stub_12, 12);
isr_err_stub!(isr_err_stub_13, 13);
isr_err_stub!(isr_err_stub_14, 14);
isr_no_err_stub!(isr_no_err_stub_15, 15);
isr_no_err_stub!(isr_no_err_stub_16, 16);
isr_err_stub!(isr_err_stub_17, 17);
isr_no_err_stub!(isr_no_err_stub_18, 18);
isr_no_err_stub!(isr_no_err_stub_19, 19);
isr_no_err_stub!(isr_no_err_stub_20, 20);
isr_no_err_stub!(isr_no_err_stub_21, 21);
isr_no_err_stub!(isr_no_err_stub_22, 22);
isr_no_err_stub!(isr_no_err_stub_23, 23);
isr_no_err_stub!(isr_no_err_stub_24, 24);
isr_no_err_stub!(isr_no_err_stub_25, 25);
isr_no_err_stub!(isr_no_err_stub_26, 26);
isr_no_err_stub!(isr_no_err_stub_27, 27);
isr_no_err_stub!(isr_no_err_stub_28, 28);
isr_no_err_stub!(isr_no_err_stub_29, 29);
isr_err_stub!(isr_err_stub_30, 30);
isr_no_err_stub!(isr_no_err_stub_31, 31);

pub fn remap_pic() {
    let pic1_command = Port::new(0x20);
    let pic1_data = Port::new(0x21);
    let pic2_command = Port::new(0xA0);
    let pic2_data = Port::new(0xA1);

    const ICW1_INIT: u8 = 0x10;
    const ICW1_ICW4: u8 = 0x01;
    const ICW4_8086: u8 = 0x01;

    unsafe {
        let a1 = pic1_data.read();
        let a2 = pic2_data.read();

        // Start initialization
        pic1_command.write(ICW1_INIT | ICW1_ICW4);
        pic2_command.write(ICW1_INIT | ICW1_ICW4);

        // Set vector offsets
        pic1_data.write(0x20); // IRQ0–7 -> INT 32–39
        pic2_data.write(0x28); // IRQ8–15 -> INT 40–47

        // Tell PICs how they are wired
        pic1_data.write(4);
        pic2_data.write(2);

        // Set 8086 mode
        pic1_data.write(ICW4_8086);
        pic2_data.write(ICW4_8086);

        // Restore saved masks
        pic1_data.write(a1);
        pic2_data.write(a2);
    }
}

static mut IDT: InterruptDescriptorTable = unsafe { core::mem::zeroed() };

pub fn set_idt() {
    unsafe {
        IDT = InterruptDescriptorTable::new();
        IDT.set_descriptor(
            0,
            InterruptDescriptor::new(isr_no_err_stub_0 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            1,
            InterruptDescriptor::new(isr_no_err_stub_1 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            2,
            InterruptDescriptor::new(isr_no_err_stub_2 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            3,
            InterruptDescriptor::new(isr_no_err_stub_3 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            4,
            InterruptDescriptor::new(isr_no_err_stub_4 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            5,
            InterruptDescriptor::new(isr_no_err_stub_5 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            6,
            InterruptDescriptor::new(isr_no_err_stub_6 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            7,
            InterruptDescriptor::new(isr_no_err_stub_7 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            8,
            InterruptDescriptor::new(isr_err_stub_8 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            9,
            InterruptDescriptor::new(isr_no_err_stub_9 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            10,
            InterruptDescriptor::new(isr_err_stub_10 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            11,
            InterruptDescriptor::new(isr_err_stub_11 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            12,
            InterruptDescriptor::new(isr_err_stub_12 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            13,
            InterruptDescriptor::new(isr_err_stub_13 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            14,
            InterruptDescriptor::new(isr_err_stub_14 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            15,
            InterruptDescriptor::new(isr_no_err_stub_15 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            16,
            InterruptDescriptor::new(isr_no_err_stub_16 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            17,
            InterruptDescriptor::new(isr_err_stub_17 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            18,
            InterruptDescriptor::new(isr_no_err_stub_18 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            19,
            InterruptDescriptor::new(isr_no_err_stub_19 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            20,
            InterruptDescriptor::new(isr_no_err_stub_20 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            21,
            InterruptDescriptor::new(isr_no_err_stub_21 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            22,
            InterruptDescriptor::new(isr_no_err_stub_22 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            23,
            InterruptDescriptor::new(isr_no_err_stub_23 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            24,
            InterruptDescriptor::new(isr_no_err_stub_24 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            25,
            InterruptDescriptor::new(isr_no_err_stub_25 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            26,
            InterruptDescriptor::new(isr_no_err_stub_26 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            27,
            InterruptDescriptor::new(isr_no_err_stub_27 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            28,
            InterruptDescriptor::new(isr_no_err_stub_28 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            29,
            InterruptDescriptor::new(isr_no_err_stub_29 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            30,
            InterruptDescriptor::new(isr_err_stub_30 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            31,
            InterruptDescriptor::new(isr_no_err_stub_31 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        for i in 31..48 {
            IDT.set_descriptor(
                i as u8,
                InterruptDescriptor::new(isr_no_err_stub_31 as usize, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
            );
        }
        remap_pic();
        IDT.load();
    }
}
