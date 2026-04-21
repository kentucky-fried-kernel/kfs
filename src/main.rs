#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::hint::spin_loop;

use kfs::{
    arch::x86::kernel_mutex::KernelMutex,
    boot::MultibootInfo,
    keyboard::{
        Keyboard,
        layout::{Layout, map_qwerty},
    },
    printkln, serial_println,
    shell::Shell,
};

mod panic;

pub const MEMORY_MAX: u64 = 1 << 32;

struct NullAllocator;

unsafe impl core::alloc::GlobalAlloc for NullAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut() // always fail
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // no-op
    }
}

#[global_allocator]
static GLOBAL: NullAllocator = NullAllocator;

static HEHE: KernelMutex<usize> = KernelMutex::new(0);
/// # Panics
/// This function will panic if initialization of dynamic memory allocation fails.
// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::arch;

    arch::x86::gdt::init();
    arch::x86::idt::init();
    arch::x86::vmm::init();

    let kernel_end: usize = &raw const _kernel_end as usize;
    printkln!("hello from late");

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
