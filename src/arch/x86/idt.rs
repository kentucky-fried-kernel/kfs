#![allow(static_mut_refs)]

use crate::panic;
const MAX_INTERRUPT_DESCRIPTORS: usize = 256;

type InterruptHandler = extern "x86-interrupt" fn(_: InterruptStackFrame);

#[repr(C)]
struct InterruptStackFrame {
    ip: u32,
    cs: u32,
    flags: u32,
    sp: u32,
    ss: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InterruptDescriptor {
    offset_1: u16,
    selector: u16,
    zero: u8,
    type_attributes: u8,
    offset_2: u16,
}

#[repr(packed)]
struct IDTR {
    pub size: u16,
    pub offset: u32,
}

#[repr(C, align(16))]
struct InterruptDescriptorTable {
    pub entries: [InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS],
    pub idtr: IDTR,
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
    pub fn new(offset: u32, selector: u16, type_attributes: u8) -> Self {
        Self {
            offset_1: (offset & 0xFFFF) as u16,
            selector,
            zero: 0,
            type_attributes,
            offset_2: ((offset >> 16) & 0xFFFF) as u16,
        }
    }

    pub fn offset(&self) -> u32 {
        ((self.offset_2 as u32) << 16) | self.offset_1 as u32
    }
}

impl InterruptDescriptorTable {
    /// Creates a new `InterruptDescriptorTable` filled with non-present entries.
    pub fn new() -> Self {
        let mut idt = Self {
            entries: [InterruptDescriptor::new(0, 0, 0); MAX_INTERRUPT_DESCRIPTORS],
            idtr: IDTR { offset: 0, size: 0 },
        };

        idt.idtr.offset = idt.entries.as_ptr() as u32;
        idt.idtr.size = (core::mem::size_of::<[InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS]>() - 1) as u16;

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
extern "C" fn exception_handler() {
    panic!("EXCEPTION");
}

macro_rules! isr_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!("call exception_handler")
        }
    };
}

macro_rules! isr_no_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!("call exception_handler")
        }
    };
}

isr_no_err_stub!(isr_no_err_stub0, 0);
isr_no_err_stub!(isr_no_err_stub1, 1);
isr_no_err_stub!(isr_no_err_stub2, 2);
isr_no_err_stub!(isr_no_err_stub3, 3);
isr_no_err_stub!(isr_no_err_stub4, 4);
isr_no_err_stub!(isr_no_err_stub5, 5);
isr_no_err_stub!(isr_no_err_stub6, 6);
isr_no_err_stub!(isr_no_err_stub7, 7);
isr_err_stub!(isr_err_stub8, 8);
isr_no_err_stub!(isr_no_err_stub9, 9);
isr_err_stub!(isr_err_stub10, 10);
isr_err_stub!(isr_err_stub11, 11);
isr_err_stub!(isr_err_stub12, 12);
isr_err_stub!(isr_err_stub13, 13);
isr_err_stub!(isr_err_stub14, 14);
isr_no_err_stub!(isr_no_err_stub15, 15);
isr_no_err_stub!(isr_no_err_stub16, 16);
isr_err_stub!(isr_err_stub17, 17);
isr_no_err_stub!(isr_no_err_stub18, 18);
isr_no_err_stub!(isr_no_err_stub19, 19);
isr_no_err_stub!(isr_no_err_stub20, 20);
isr_no_err_stub!(isr_no_err_stub21, 21);
isr_no_err_stub!(isr_no_err_stub22, 22);
isr_no_err_stub!(isr_no_err_stub23, 23);
isr_no_err_stub!(isr_no_err_stub24, 24);
isr_no_err_stub!(isr_no_err_stub25, 25);
isr_no_err_stub!(isr_no_err_stub26, 26);
isr_no_err_stub!(isr_no_err_stub27, 27);
isr_no_err_stub!(isr_no_err_stub28, 28);
isr_no_err_stub!(isr_no_err_stub29, 29);
isr_err_stub!(isr_err_stub30, 30);
isr_no_err_stub!(isr_no_err_stub31, 31);

static mut IDT: InterruptDescriptorTable = unsafe { core::mem::zeroed() };

pub fn set_idt() {
    unsafe {
        IDT = InterruptDescriptorTable::new();
        IDT.set_descriptor(
            0,
            InterruptDescriptor::new(isr_no_err_stub0 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            1,
            InterruptDescriptor::new(isr_no_err_stub1 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            2,
            InterruptDescriptor::new(isr_no_err_stub2 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            3,
            InterruptDescriptor::new(isr_no_err_stub3 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            4,
            InterruptDescriptor::new(isr_no_err_stub4 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            5,
            InterruptDescriptor::new(isr_no_err_stub5 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            6,
            InterruptDescriptor::new(isr_no_err_stub6 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            7,
            InterruptDescriptor::new(isr_no_err_stub7 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            8,
            InterruptDescriptor::new(isr_err_stub8 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            9,
            InterruptDescriptor::new(isr_no_err_stub9 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            10,
            InterruptDescriptor::new(isr_err_stub10 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            11,
            InterruptDescriptor::new(isr_err_stub11 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            12,
            InterruptDescriptor::new(isr_err_stub12 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            13,
            InterruptDescriptor::new(isr_err_stub13 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            14,
            InterruptDescriptor::new(isr_err_stub14 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            15,
            InterruptDescriptor::new(isr_no_err_stub15 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            16,
            InterruptDescriptor::new(isr_no_err_stub16 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            17,
            InterruptDescriptor::new(isr_err_stub17 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            18,
            InterruptDescriptor::new(isr_no_err_stub18 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            19,
            InterruptDescriptor::new(isr_no_err_stub19 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            20,
            InterruptDescriptor::new(isr_no_err_stub20 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            21,
            InterruptDescriptor::new(isr_no_err_stub21 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            22,
            InterruptDescriptor::new(isr_no_err_stub22 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            23,
            InterruptDescriptor::new(isr_no_err_stub23 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            24,
            InterruptDescriptor::new(isr_no_err_stub24 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            25,
            InterruptDescriptor::new(isr_no_err_stub25 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            26,
            InterruptDescriptor::new(isr_no_err_stub26 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            27,
            InterruptDescriptor::new(isr_no_err_stub27 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            28,
            InterruptDescriptor::new(isr_no_err_stub28 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            29,
            InterruptDescriptor::new(isr_no_err_stub29 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            30,
            InterruptDescriptor::new(isr_err_stub30 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );
        IDT.set_descriptor(
            31,
            InterruptDescriptor::new(isr_no_err_stub31 as u32, 0x08, build_type_attributes(1, 0, GateType::InterruptGate32)),
        );

        IDT.load();
    }
}
