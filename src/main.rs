#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{arch::naked_asm, hint::spin_loop, ptr::write_volatile};

use kfs::{
    arch::x86::{idt::InterruptRegisters, interrupts::irq, vmm::mmap_init},
    boot::MultibootInfo,
    printkln, scheduling,
};

mod panic;

pub const MEMORY_MAX: u64 = 1 << 32;

unsafe extern "C" {
    static _kernel_end: u8;
}
/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::arch;

    arch::x86::gdt::init();
    printkln!("Gdt initialized");
    arch::x86::idt::init();
    printkln!("Idt initialized");
    arch::x86::vmm::init(info);
    printkln!("Vmm initialized");
    scheduling::init();
    printkln!("Scheduling initialized");

    loop {
        spin_loop();
    }
    // kfs::ps2::init();
}

// enum ExecveError {
//     CouldNotAllocate,
// }
//
// unsafe fn execve(binary: &Binary) -> Result<(), ExecveError> {
//     for s in binary.segments.iter() {
//         mmap_init(s.vaddr as *mut u8, None, s.size, 1);
//
//         unsafe {
//             core::ptr::copy_nonoverlapping(s.offset as *mut u8, s.vaddr as *mut u8, s.size);
//         }
//     }
//
//     irq::install_handler(0, timer);
//     irq::clear_mask(0);
//
//     unsafe {
//         jump_to_userspace(binary.entry, binary.stack);
//     }
//
//     Err(ExecveError::CouldNotAllocate)
// }
// unsafe fn jump_to_userspace(entry: usize, user_stack: usize) {
//     kfs::serial_println!("jump into");
//     unsafe {
//         core::arch::asm!(
//             // iret frame (pushed in reverse order)
//             "mov ax, 0x23",   // user data | RPL 3
//             "mov ds, ax",
//             "mov es, ax",
//             "mov fs, ax",
//             "mov gs, ax",
//
//             "push 0x23",      // SS  — 0x28 | 3
//             "push {esp}",     // ESP
//             "push 0x200",     // EFLAGS — IF set
//             "push 0x1B",      // CS  — 0x20 | 3
//             "push {eip}",     // EIP
//             "iretd",
//             esp = in(reg) user_stack,
//             eip = in(reg) entry,
//             options(noreturn)
//         );
//     }
// }
//
//
// #[unsafe(no_mangle)]
// extern "C" fn timer(regs: &InterruptRegisters) {
//     kfs::serial_println!("timer");
//     kfs::serial_println!("{:x}", regs.eip);
//     kfs::serial_println!("{:x}", regs.eax);
// }

/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu};

    arch::x86::gdt::init();
    printkln!("Gdt initialized");
    arch::x86::idt::init();
    printkln!("Idt initialized");
    arch::x86::vmm::init(info);
    printkln!("Vmm initialized");

    test_main();

    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
