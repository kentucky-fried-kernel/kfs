#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(test))]
use kfs::boot::MultibootInfo;
use kfs::{
    boot::{INITIAL_PAGE_DIR, KERNEL_BASE},
    printkln,
};

mod panic;

fn invalidate(vaddr: usize) {
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vaddr) };
}

#[allow(static_mut_refs)]
fn init_memory(mem_high: usize, physical_alloc_start: usize) {
    // We do not need the identity mapped kernel anymore, so we can remove
    // its PD entry.
    unsafe { INITIAL_PAGE_DIR[0] = 0 };
    invalidate(0);

    let page_dir_phys = unsafe { (&INITIAL_PAGE_DIR as *const _ as usize) - KERNEL_BASE };
    printkln!("page_dir_phys: 0x{:x}", page_dir_phys);
    unsafe { INITIAL_PAGE_DIR[1023] = page_dir_phys | 1 | 2 };
    invalidate(0xFFFFF000);
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn kmain(magic: usize, info: &MultibootInfo) {
    use kfs::{arch, printkln, shell, terminal};

    printkln!("Multiboot Magic: {:x}", magic);
    printkln!("Multiboot Info :  {}", info);
    printkln!("_start       : 0x{:x}", kfs::boot::_start as *const () as usize);
    printkln!("kmain        : 0x{:x}", kmain as *const () as usize);
    printkln!("idt::init    : 0x{:x}", arch::x86::idt::init as *const () as usize);
    printkln!("higher_half  : 0x{:x}", kfs::boot::higher_half as *const () as usize);

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    use kfs::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
