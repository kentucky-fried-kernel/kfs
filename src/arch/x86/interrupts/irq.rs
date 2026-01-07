use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        interrupts::{
            lock::IRQLock,
            pic::{self, PIC1_DATA, PIC2_DATA},
        },
    },
    port::Port,
};

macro_rules! stub {
    ($func: ident, $nb: expr, $val: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push 0", "push {}", "jmp irq_common_stub", const $val);
        }
    };
}

stub!(irq_stub_0, 0, 32);
stub!(irq_stub_1, 1, 33);
stub!(irq_stub_2, 2, 34);
stub!(irq_stub_3, 3, 35);
stub!(irq_stub_4, 4, 36);
stub!(irq_stub_5, 5, 37);
stub!(irq_stub_6, 6, 38);
stub!(irq_stub_7, 7, 39);
stub!(irq_stub_8, 8, 40);
stub!(irq_stub_9, 9, 41);
stub!(irq_stub_10, 10, 42);
stub!(irq_stub_11, 11, 43);
stub!(irq_stub_12, 12, 44);
stub!(irq_stub_13, 13, 45);
stub!(irq_stub_14, 14, 46);
stub!(irq_stub_15, 15, 47);

#[macro_export]
macro_rules! irq_stubs {
    () => {
        &[
            $crate::arch::x86::interrupts::irq::irq_stub_0 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_1 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_2 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_3 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_4 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_5 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_6 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_7 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_8 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_9 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_10 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_11 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_12 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_13 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_14 as *const () as usize,
            $crate::arch::x86::interrupts::irq::irq_stub_15 as *const () as usize,
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
        "iretd"
    )
}

static mut IRQ_ROUTINES: [Option<extern "C" fn(&InterruptRegisters)>; 16] = [None; 16];

/// Installs a handler for `irq`. Note that this does not unmask `irq`, it should be done
/// explicitly by the caller.
#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub fn install_handler(irq: u32, handler: extern "C" fn(&InterruptRegisters)) {
    let _lock = IRQLock::lock(irq as u8);
    // SAFETY:
    // We are mutating IRQ_ROUTINES, which we know is valid for the entire lifetime of the program, and
    // will not be modified by any other part of the kernel.
    unsafe { IRQ_ROUTINES[irq as usize] = Some(handler) };
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
unsafe fn uninstall_handler(irq: u32) {
    let _lock = IRQLock::lock(irq as u8);
    // SAFETY:
    // We are mutating IRQ_ROUTINES, which we know is valid for the entire lifetime of the program, and
    // will not be modified by any other part of the kernel.
    unsafe { IRQ_ROUTINES[irq as usize] = None };
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
unsafe extern "C" fn irq_handler(regs: &InterruptRegisters) {
    #[allow(clippy::cast_possible_wrap)]
    let irq_index = if regs.intno as isize - 32 < 0 {
        return;
    } else {
        (regs.intno - 32) as usize
    };

    // SAFETY:
    // We are accessing IRQ_ROUTINES, which we know is valid for the entire lifetime of the program and
    // will not be accessed concurrently by any other part of the kernel.
    if let Some(handler) = unsafe { IRQ_ROUTINES[irq_index] } {
        handler(regs);
    };

    pic::send_eoi(irq_index as u8);
}

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
