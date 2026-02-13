#![allow(unused_imports)]
use core::panic::PanicInfo;

use kfs::terminal::entry::Color;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kfs::{cli, hlt, printkln, serial_println};

    cli!();

    printkln!("KERNEL PANIC: {:?}\n", info.message());

    serial_println!("KERNEL PANIC: {:?}", info.message());

    clear_regs!();
    hlt!();
}

#[macro_export]
macro_rules! clear_regs {
    () => {
        core::arch::asm!(
            "xor rax, rax",
            "xor rbx, rbx",
            "xor rcx, rcx",
            "xor rdx, rdx",
            "xor rsi, rsi",
            "xor rdi, rdi",
            "xor r8, r8",
            "xor r9, r9",
            "xor r10, r10",
            "xor r11, r11",
            "xor r12, r12",
            "xor r13, r13",
            "xor r14, r14",
            "xor r15, r15",
            options(nomem, preserves_flags)
        );
    };
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info);
}
