#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::{
    arch::x86::{idt::InterruptRegisters, interrupts::irq},
    boot::MultibootInfo,
    keyboard::{
        Keyboard,
        layout::{Layout, map_qwerty},
    },
    serial_println,
    shell::Shell,
    vmm::paging::{
        PAGE_SIZE,
        mmap::{Mode, mmap},
    },
};

mod panic;

extern crate alloc;

/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch,
        vmm::{self, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info);

    kfs::ps2::init();

    // if vmm::allocators::kmalloc::init().is_err() {
    //     panic!("Failed to initialize kmalloc");
    // }

    let mode = Mode::Continous;
    let space = mmap(None, PAGE_SIZE, vmm::paging::Permissions::ReadWrite, vmm::paging::Access::User, &mode).unwrap();
    let space = mmap(None, PAGE_SIZE, vmm::paging::Permissions::ReadWrite, vmm::paging::Access::User, &mode).unwrap();

    kfs::serial_println!("space {:?}", space);

    irq::install_handler(0, timer);

    irq::clear_mask(0);

    let mode = Mode::Scattered;
    let user_program = mmap(None, PAGE_SIZE * 100, vmm::paging::Permissions::ReadWrite, vmm::paging::Access::User, &mode).unwrap();
    unsafe {
        core::ptr::copy_nonoverlapping(user_function as *const u8, user_program as *mut u8, 100);
    }
    let user_stack = mmap(None, PAGE_SIZE * 100, vmm::paging::Permissions::ReadWrite, vmm::paging::Access::User, &mode).unwrap();
    jump_to_userspace(user_program as u32, user_stack as u32 + PAGE_SIZE as u32 * 10);

    loop {}
    // #[allow(static_mut_refs)]
    // let mut shell = Shell::default(unsafe { &mut kfs::terminal::SCREEN },
    // Keyboard::new(Layout::new(map_qwerty))); shell.launch();
}

#[unsafe(no_mangle)]
extern "C" fn timer(regs: &InterruptRegisters) {
    kfs::serial_println!("timer");
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn user_function(intno: u32, stack_ptr: u32) {
    core::arch::naked_asm!("aaaa:", "jmp aaaa",)
}

pub fn jump_to_userspace(entry: u32, user_stack: u32) {
    serial_println!("jump into");
    unsafe {
        core::arch::asm!(
            // iret frame (pushed in reverse order)
            "mov ax, 0x2B",   // user data | RPL 3
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",

            "push 0x2B",      // SS  — 0x28 | 3
            "push {esp}",     // ESP
            "push 0x200",     // EFLAGS — IF set
            "push 0x23",      // CS  — 0x20 | 3
            "push {eip}",     // EIP
            "iretd",
            esp = in(reg) user_stack,
            eip = in(reg) entry,
            options(noreturn)
        );
    }
}

//
// /// # Panics
// /// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(test)]
// #[unsafe(no_mangle)]
// pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
//     use kfs::{arch, qemu, vmm};
//
//     arch::x86::gdt::init();
//     arch::x86::idt::init();
//
//     vmm::paging::init::init_memory(info);
//
//     kfs::ps2::init();
//
//     if vmm::allocators::kmalloc::init().is_err() {
//         panic!("Failed to initialize kmalloc");
//     }
//
//     test_main();
//
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
