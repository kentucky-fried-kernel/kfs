#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod panic;

const STACK_SIZE: usize = 2 << 20;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss")]
pub static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[allow(unused)]
#[repr(align(4096))]
pub struct Stack([u8; STACK_SIZE]);

#[allow(unused)]
#[repr(align(4))]
struct MultibootHeader {
    magic: usize,
    flags: usize,
    checksum: usize,
}

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot")]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: 0x1badb002,
    flags: 0,
    checksum: (0usize.wrapping_sub(0x1badb002 + 0)),
};

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        "mov esp, offset STACK + {stack_size}",
        "push eax",
        "push ebx",
        "cli",
        "call kernel_main",
        "hang:",
        "cli",
        "hlt",
        "jmp hang",
        stack_size = const STACK_SIZE
    )
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use kfs::{arch, ps2, shell, terminal};
    if let Err(e) = ps2::init() {
        panic!("could not initialize PS/2: {}", e);
    }
    arch::x86::gdt::init();
    #[cfg(not(test))]
    arch::x86::set_idt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use kfs::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
