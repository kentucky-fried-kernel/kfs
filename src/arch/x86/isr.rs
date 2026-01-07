use crate::{arch::x86::idt::InterruptRegisters, serial_println};

macro_rules! no_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push 0", "push {0}", "jmp isr_common_stub", const $nb);
        }
    };
}

macro_rules! err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push {0}", "jmp isr_common_stub", const $nb);
        }
    };
}

pub mod _stubs {
    no_err_stub!(isr_stub_0, 0);
    no_err_stub!(isr_stub_1, 1);
    no_err_stub!(isr_stub_2, 2);
    no_err_stub!(isr_stub_3, 3);
    no_err_stub!(isr_stub_4, 4);
    no_err_stub!(isr_stub_5, 5);
    no_err_stub!(isr_stub_6, 6);
    no_err_stub!(isr_stub_7, 7);
    err_stub!(isr_stub_8, 8);
    no_err_stub!(isr_stub_9, 9);
    err_stub!(isr_stub_10, 10);
    err_stub!(isr_stub_11, 11);
    err_stub!(isr_stub_12, 12);
    err_stub!(isr_stub_13, 13);
    err_stub!(isr_stub_14, 14);
    no_err_stub!(isr_stub_15, 15);
    no_err_stub!(isr_stub_16, 16);
    err_stub!(isr_stub_17, 17);
    no_err_stub!(isr_stub_18, 18);
    no_err_stub!(isr_stub_19, 19);
    no_err_stub!(isr_stub_20, 20);
    no_err_stub!(isr_stub_21, 21);
    no_err_stub!(isr_stub_22, 22);
    no_err_stub!(isr_stub_23, 23);
    no_err_stub!(isr_stub_24, 24);
    no_err_stub!(isr_stub_25, 25);
    no_err_stub!(isr_stub_26, 26);
    no_err_stub!(isr_stub_27, 27);
    no_err_stub!(isr_stub_28, 28);
    no_err_stub!(isr_stub_29, 29);
    err_stub!(isr_stub_30, 30);
    no_err_stub!(isr_stub_31, 31);
}

#[macro_export]
macro_rules! isr_stubs {
    () => {
        &[
            crate::arch::x86::isr::_stubs::isr_stub_0 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_1 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_2 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_3 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_4 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_5 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_6 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_7 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_8 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_9 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_10 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_11 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_12 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_13 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_14 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_15 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_16 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_17 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_18 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_19 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_20 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_21 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_22 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_23 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_24 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_25 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_26 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_27 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_28 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_29 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_30 as *const () as usize,
            crate::arch::x86::isr::_stubs::isr_stub_31 as *const () as usize,
        ]
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
        "iretd"
    )
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
unsafe extern "C" fn isr_handler(regs: &InterruptRegisters) {
    if regs.intno < 32 {
        serial_println!("\nINT {}: {}", regs.intno, INTERRUPT_MESSAGE[regs.intno as usize]);
    }
}
