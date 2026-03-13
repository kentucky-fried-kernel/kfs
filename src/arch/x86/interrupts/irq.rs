//! # IRQ (Interrupt Request) Handling
//!
//! This module handles hardware interrupts (IRQs) from the PIC (Programmable Interrupt Controller).
//!
//! ## Why we use `jmp eax` instead of `iretd` for context switching
//!
//! When an interrupt occurs through an **Interrupt Gate** (as configured in the IDT), the CPU
//! automatically **clears the IF (Interrupt Flag)** in EFLAGS before jumping to the handler.
//! This means interrupts are disabled during interrupt handling.
//!
//! The `iretd` instruction restores EIP, CS, and EFLAGS from the stack. The problem is:
//!
//! 1. When we save a task's context during an interrupt, the saved EFLAGS has **IF=0** (interrupts
//!    disabled) because the CPU cleared it when entering the interrupt.
//!
//! 2. When context switching to a new task, if we use `iretd` to restore that task's saved EFLAGS
//!    (with IF=0), **interrupts remain disabled**.
//!
//! 3. With interrupts disabled, the timer interrupt never fires again, so no more context switches
//!    occur - the system appears to "stop".
//!
//! ### The Solution
//!
//! Instead of using `iretd`, we manually:
//! 1. Pop EIP into a register (`pop eax`)
//! 2. Skip over CS and EFLAGS on the stack (`add esp, 8`)
//! 3. Explicitly re-enable interrupts (`sti`)
//! 4. Jump to the new EIP (`jmp eax`)
//!
//! This ensures interrupts are always re-enabled after a context switch, regardless of
//! the saved EFLAGS value.
//!
//! ### Alternative Fix
//!
//! If you prefer using `iretd`, you must ensure the saved EFLAGS has **IF=1** when
//! creating the initial context for a new task. You can do this by OR-ing the EFLAGS
//! value with `0x200` (the IF bit) when setting up the task's stack frame.

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

/// Common IRQ handler stub that saves/restores context and supports context switching.
///
/// # Context Switching Mechanism
///
/// The `irq_handler` can return either:
/// - `0`: No context switch needed, continue with current task
/// - Non-zero: New stack pointer for context switch to a different task
///
/// When a context switch occurs, ESP is changed to point to the new task's saved
/// context, and execution continues from there.
#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn irq_common_stub(intno: u32, stack_ptr: u32) {
    core::arch::naked_asm!(
        // Save all general-purpose registers
        "pusha",
        // Save data segment selector
        "mov eax, ds",
        "push eax",
        // Save CR2 (page fault address, useful for debugging)
        "mov eax, cr2",
        "push eax",
        // Switch to kernel data segment (0x10)
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        // Call the Rust IRQ handler with pointer to saved registers
        // Handler returns 0 for no context switch, or new ESP for context switch
        "push esp",
        "call irq_handler",
        // Check if context switch is requested (eax != 0)
        "test eax, eax",
        "jz after",
        // Context switch: set ESP to the new task's saved context
        "mov esp, eax",
        "after:",
        // Skip cr2 (not restored)
        "add esp, 4",
        // Restore data segment selector
        "pop ebx",
        "mov ebx, 0x10",
        "mov ds, bx",
        "mov es, bx",
        "mov fs, bx",
        "mov gs, bx",
        // Restore general-purpose registers
        "popa",
        // Skip error code and interrupt number
        "add esp, 8",
        // IMPORTANT: We cannot use iretd here because the saved EFLAGS has IF=0
        // (interrupts disabled), since the CPU clears IF when entering via an
        // Interrupt Gate. Using iretd would restore IF=0, leaving interrupts
        // disabled and preventing further timer interrupts/context switches.
        //
        // Instead, we manually:
        // 1. Pop EIP into eax
        // 2. Skip CS and EFLAGS (add esp, 8)
        // 3. Explicitly enable interrupts (sti)
        // 4. Jump to the saved EIP
        "pop eax",
        "add esp, 8",
        "sti",
        "jmp eax",
    )
}

static mut IRQ_ROUTINES: [Option<extern "C" fn(*mut InterruptRegisters) -> u32>; 16] = [None; 16];

/// Installs a handler for `irq`. Note that this does not unmask `irq`, it should be done
/// explicitly by the caller.
#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub fn install_handler(irq: u32, handler: extern "C" fn(*mut InterruptRegisters) -> u32) {
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

/// IRQ handler that dispatches to registered handlers and supports context switching.
///
/// # Returns
/// - `0`: No context switch needed
/// - Non-zero: New ESP value pointing to the new task's saved context
#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
unsafe extern "C" fn irq_handler(regs: &mut InterruptRegisters) -> u32 {
    #[allow(clippy::cast_possible_wrap)]
    let irq_index = if regs.intno as isize - 32 < 0 {
        return 0;
    } else {
        (regs.intno - 32) as usize
    };

    let mut res = 0;
    // SAFETY:
    // We are accessing IRQ_ROUTINES, which we know is valid for the entire lifetime of the program and
    // will not be accessed concurrently by any other part of the kernel.
    if let Some(handler) = unsafe { IRQ_ROUTINES[irq_index] } {
        res = handler(regs);
    };

    pic::send_eoi(irq_index as u8);
    res
}

/// # Panics
/// This panics if irq_line is bigger than 15
pub fn set_mask(mut irq_line: u8) {
    assert!(irq_line < 16);
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

/// # Panics
/// This panics if irq_line is bigger than 15
pub fn clear_mask(mut irq_line: u8) {
    assert!(irq_line < 16);
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
