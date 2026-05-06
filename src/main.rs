#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{
    alloc,
    hint::spin_loop,
    ptr::{read_volatile, write_volatile},
};

use kfs::{boot::MultibootInfo, printkln};

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
    use kfs::arch::{self, x86::vmm::addressspace::Addressspace};

    arch::x86::gdt::init();
    printkln!("Gdt initialized");
    arch::x86::idt::init();
    printkln!("Idt initialized");
    arch::x86::vmm::init(info);
    printkln!("Vmm initialized");
    arch::x86::scheduler::init();
    printkln!("Scheduler initialized");

    printkln!("Booted");

    // let mut a = Addressspace::new();
    // a.map(0x2000 as *mut u8, 0x2000_0000 as *mut u8, 1);
    // a.load();
    //
    // let b = 3;
    // unsafe {
    //     write_volatile(0x2000 as *mut u8, b);
    // }
    //
    // unsafe {
    //     kfs::serial_println!("{}", read_volatile(0x2000 as *mut u8));
    // }
    // a.map(0x2000 as *mut u8, 0x2000_1000 as *mut u8, 1);
    // unsafe {
    //     kfs::serial_println!("{}", read_volatile(0x2000 as *mut u8));
    // }
    loop {
        spin_loop();
    }
    // kfs::ps2::init();
}
//
// /// # Panics
// /// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(test)]
// #[unsafe(no_mangle)]
// pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
//     use kfs::{arch, qemu};
//
//     arch::x86::gdt::init();
//     printkln!("Gdt initialized");
//     arch::x86::idt::init();
//     printkln!("Idt initialized");
//     arch::x86::vmm::init(info);
//     printkln!("Vmm initialized");
//
//     test_main();
//
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
