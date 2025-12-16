#![allow(unused)]
#![allow(static_mut_refs)]
#![allow(clippy::cast_possible_truncation)]

use crate::{
    arch::x86::{
        gdt::KERNEL_CODE_OFFSET,
        pic::{self, send_eoi},
    },
    printk, printkln, serial_println,
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

macro_rules! isr_no_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push 0", "push {0}", "jmp isr_common_stub", const $nb);
        }
    };
}

macro_rules! isr_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push {0}", "jmp isr_common_stub", const $nb);
        }
    };
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn isr_common_stub(intno: u32, stack_ptr: u32) {
    core::arch::naked_asm!(
        "pusha",
        "mov eax, ds",
        "push eax",
        "mov eax, cr2",
        "push eax",
        //
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        //
        "push esp",
        "call isr_handler",
        //
        "add esp, 8",
        "pop ebx",
        "mov ds, bx",
        "mov es, bx",
        "mov fs, bx",
        "mov gs, bx",
        //
        "popa",
        "add esp, 8",
        "sti",
        "iret"
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptRegisters {
    cr2: u32,
    ds: u32,
    edi: u32,
    esi: u32,
    ebp: u32,
    esp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    intno: u32,
    err_code: u32,
    eip: u32,
    csm: u32,
    eflags: u32,
    useresp: u32,
    ss: u32,
}

const INTERRUPT_MESSAGE: &[&str] = &[
    "Division By Zero",
    "Debug Interrupt",
    "Non-Maskable Interrupt",
    "Breakpoint",
    "Into Detected Overflow",
    "Out of Bounds",
    "Invalid Opcode",
    "No Coprocessor",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Bad TSS",
    "Segment not Present",
    "Stack Fault",
    "General Protection Fault",
    "Page Fault",
    "Unknown Interrupt",
    "Coprocessor Fault",
    "Aligment Fault",
    "Machine Check",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
];

#[unsafe(no_mangle)]
unsafe extern "C" fn isr_handler(regs: *const InterruptRegisters) {
    // SAFETY:
    // The address of `regs` is pushed onto the stack by `isr_common_stub`.
    let regs = unsafe { &*regs };
    serial_println!("isr_handler: {:?}", regs);
    if regs.intno < 32 {
        printkln!("\nGot Interrupt {}: {}", regs.intno, INTERRUPT_MESSAGE[regs.intno as usize]);
        printkln!("Exception: System Halted\n");
        // SAFETY:
        // We are using inline assembly to halt the system.
        unsafe { core::arch::asm!("cli", "hlt") };
    } else {
        panic!("Got unknown interrupt");
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
        ]
    };
}

macro_rules! irq_stub {
    ($func: ident, $nb: expr, $val: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push 0", "push {}", "jmp irq_common_stub", const $val);
        }
    };
}

irq_stub!(irq_stub_0, 0, 32);
irq_stub!(irq_stub_1, 1, 33);
irq_stub!(irq_stub_2, 2, 34);
irq_stub!(irq_stub_3, 3, 35);
irq_stub!(irq_stub_4, 4, 36);
irq_stub!(irq_stub_5, 5, 37);
irq_stub!(irq_stub_6, 6, 38);
irq_stub!(irq_stub_7, 7, 39);
irq_stub!(irq_stub_8, 8, 40);
irq_stub!(irq_stub_9, 9, 41);
irq_stub!(irq_stub_10, 10, 42);
irq_stub!(irq_stub_11, 11, 43);
irq_stub!(irq_stub_12, 12, 44);
irq_stub!(irq_stub_13, 13, 45);
irq_stub!(irq_stub_14, 14, 46);
irq_stub!(irq_stub_15, 15, 47);

macro_rules! irq_stubs {
    () => {
        &[
            irq_stub_0 as *const () as usize,
            irq_stub_1 as *const () as usize,
            irq_stub_2 as *const () as usize,
            irq_stub_3 as *const () as usize,
            irq_stub_4 as *const () as usize,
            irq_stub_5 as *const () as usize,
            irq_stub_6 as *const () as usize,
            irq_stub_7 as *const () as usize,
            irq_stub_8 as *const () as usize,
            irq_stub_9 as *const () as usize,
            irq_stub_10 as *const () as usize,
            irq_stub_11 as *const () as usize,
            irq_stub_12 as *const () as usize,
            irq_stub_13 as *const () as usize,
            irq_stub_14 as *const () as usize,
            irq_stub_15 as *const () as usize,
        ]
    };
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn irq_common_stub(intno: u32, stack_ptr: u32) {
    core::arch::naked_asm!(
        "pusha",
        "mov eax, ds",
        "push eax",
        "mov eax, cr2",
        "push eax",
        //
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        //
        "push esp",
        "call irq_handler",
        //
        "add esp, 8",
        "pop ebx",
        "mov ds, bx",
        "mov es, bx",
        "mov fs, bx",
        "mov gs, bx",
        //
        "popa",
        "add esp, 8",
        "sti",
        "iret"
    )
}

static mut IRQ_ROUTINES: [Option<extern "C" fn(InterruptRegisters)>; 16] = [None; 16];

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
unsafe extern "C" fn irq_install_handler(irq: u32, handler: extern "C" fn(InterruptRegisters)) {
    // SAFETY:
    // We are mutating IRQ_ROUTINES, which we know is valid for the entire lifetime of the program, and
    // will not be modified by any other part of the kernel.
    unsafe { IRQ_ROUTINES[irq as usize] = Some(handler) };
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
unsafe extern "C" fn irq_uninstall_handler(irq: u32) {
    // SAFETY:
    // We are mutating IRQ_ROUTINES, which we know is valid for the entire lifetime of the program, and
    // will not be modified by any other part of the kernel.
    unsafe { IRQ_ROUTINES[irq as usize] = None };
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
unsafe extern "C" fn irq_handler(regs: *const InterruptRegisters) {
    // SAFETY:
    // The address of `regs` is pushed onto the stack by `irq_common_stub`.
    let regs = unsafe { &*regs };
    serial_println!("irq_handler: {:?}", regs);

    #[allow(clippy::cast_possible_wrap)]
    let irq_index = if regs.intno as isize - 32 < 0 {
        printkln!("Got unhandled IRQ code {}", regs.intno);
        return;
    } else {
        (regs.intno - 32) as usize
    };

    // SAFETY:
    // We are accessing IRQ_ROUTINES, which we know is valid for the entire lifetime of the program.
    let handler = match unsafe { IRQ_ROUTINES[irq_index] } {
        Some(handler) => handler,
        None => {
            serial_println!("Unhandled IRQ");
            return;
        }
    };

    let intno = regs.intno;

    handler(*regs);

    pic::send_eoi(intno as u8);
}

static mut IDT: Option<InterruptDescriptorTable> = None;

pub fn init() {
    pic::remap(0x20, 0x28);

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
