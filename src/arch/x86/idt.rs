#![allow(unused)]
#![allow(static_mut_refs)]
#![allow(clippy::cast_possible_truncation)]

use crate::{
    arch::x86::{
        gdt::KERNEL_CODE_OFFSET,
        irq, isr,
        pic::{self, send_eoi},
    },
    irq_stubs, isr_stubs, printk, printkln, serial_println,
};

const MAX_INTERRUPT_DESCRIPTORS: usize = 256;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct InterruptDescriptor {
    isr_low: u16,
    kernel_cs: u16,
    zero: u8,
    attributes: Attributes,
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
    None = 0,
    TaskGate = 0b0101,
    InterruptGate16 = 0b0110,
    TrapGate16 = 0b0111,
    InterruptGate32 = 0b1110,
    TrapGate32 = 0b1111,
}

#[repr(u8)]
enum PrivilegeLevel {
    KernelMode = 0,
    _Ring1 = 1,
    _Ring2 = 2,
    UserMode = 3,
}

#[repr(u8)]
enum PresentBit {
    Absent = 0,
    Present = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Attributes(u8);

impl Attributes {
    fn new(present: PresentBit, dpl: PrivilegeLevel, gate_type: GateType) -> Self {
        Self(((present as u8) << 7) | ((dpl as u8) << 5) | gate_type as u8)
    }

    fn empty() -> Self {
        Self(0)
    }
}

impl From<Attributes> for u8 {
    fn from(value: Attributes) -> Self {
        value.0
    }
}

impl InterruptDescriptor {
    fn new(offset: usize, kernel_cs: u16, attributes: Attributes) -> Self {
        Self {
            isr_low: (offset & 0xFFFF) as u16,
            kernel_cs,
            zero: 0,
            attributes,
            isr_high: ((offset >> 16) & 0xFFFF) as u16,
        }
    }

    fn offset(&self) -> u32 {
        (u32::from(self.isr_high) << 16) | u32::from(self.isr_low)
    }
}

impl InterruptDescriptorTable {
    /// Creates a new `InterruptDescriptorTable` filled with non-present
    /// entries.
    fn new() -> Self {
        let mut idt = Self {
            entries: [InterruptDescriptor::new(0, 0, Attributes::empty()); MAX_INTERRUPT_DESCRIPTORS],
            idtr: InterruptDescriptorTableRegister { base: 0, limit: 0 },
        };

        idt.idtr.base = idt.entries.as_ptr() as usize;
        idt.idtr.limit = (core::mem::size_of::<[InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS]>() - 1) as u16;

        idt
    }

    fn load(&self) {
        let idtr = InterruptDescriptorTableRegister {
            base: self.entries.as_ptr() as usize,
            limit: (core::mem::size_of::<[InterruptDescriptor; MAX_INTERRUPT_DESCRIPTORS]>() - 1) as u16,
        };

        // SAFETY:
        // We are using inline assembly to get access to the `lidt` instruction. The
        // value we pass to it contains the address to a static IDT, which is
        // guaranteed stay valid for the entire lifetime of the program.
        unsafe {
            core::arch::asm!("lidt [{}]" , "sti" , in(reg) &raw const idtr, options(readonly, nostack, preserves_flags));
        }
    }

    fn set_descriptor(&mut self, index: u8, descriptor: InterruptDescriptor) {
        self.entries[index as usize] = descriptor;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptRegisters {
    pub cr2: u32,
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub intno: u32,
    pub err_code: u32,
    pub eip: u32,
    pub csm: u32,
    pub eflags: u32,
    pub useresp: u32,
    pub ss: u32,
}

static mut IDT: Option<InterruptDescriptorTable> = None;

pub fn init() {
    pic::remap(0x20, 0x28);
    pic::disable();

    let isr_stubs = isr_stubs!();
    let irq_stubs = irq_stubs!();

    let mut idt = InterruptDescriptorTable::new();

    for (index, stub) in isr_stubs.iter().enumerate() {
        idt.set_descriptor(
            index as u8,
            InterruptDescriptor::new(
                *stub,
                KERNEL_CODE_OFFSET as u16,
                Attributes::new(PresentBit::Present, PrivilegeLevel::KernelMode, GateType::InterruptGate32),
            ),
        );
    }

    for (index, stub) in irq_stubs.iter().enumerate() {
        idt.set_descriptor(
            index as u8 + 32,
            InterruptDescriptor::new(
                *stub,
                KERNEL_CODE_OFFSET as u16,
                Attributes::new(PresentBit::Present, PrivilegeLevel::KernelMode, GateType::InterruptGate32),
            ),
        );
    }

    // SAFETY:
    // The AtomicBool guarding the IDT static ensures we initialize it exactly once.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        IDT = Some(idt);

        if let Some(ref idt) = IDT {
            idt.load();
        }
    }
}
