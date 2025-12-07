#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use kfs::boot::MultibootInfo;

use kfs::{
    tester,
    vmm::{
        self,
        allocators::{backend::buddy::BUDDY_ALLOCATOR_SIZE, kmalloc::kmalloc},
    },
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    tester::should_panic_panic_handler();
}

#[test_case]
fn alloc_over_max_amount() -> Result<(), &'static str> {
    let _ = kmalloc(BUDDY_ALLOCATOR_SIZE * 2).map_err(|_| "Allocation failed")?;

    Ok(())
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm::paging::init::init_memory};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    if vmm::allocators::kmalloc::init().is_err() {
        panic!("Failed to initialize kmalloc");
    }

    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
