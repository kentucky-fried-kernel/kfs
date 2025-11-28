#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::asm;

use kfs::{boot::MultibootInfo, printk, vmm::paging::init::init_memory};

mod panic;

#[derive(Copy, Clone)]
struct Range {
    pub start: u32,
    pub size: u32,
}

impl Range {
    fn new() -> Self {
        Range { start: 0, size: 0 }
    }
}

// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(magic: usize, info: &MultibootInfo) {
    use kfs::{arch, printkln, shell, terminal, vmm};

    printkln!("Multiboot Info Struct Address: 0x{:x}", info as *const _ as usize);
    printkln!("Multiboot Magic: {:x}", magic);
    printkln!("{}", info);
    printkln!("_start       : 0x{:x}", kfs::boot::_start as *const () as usize);
    printkln!("kmain        : 0x{:x}", kmain as *const () as usize);
    printkln!("idt::init    : 0x{:x}", arch::x86::idt::init as *const () as usize);
    printkln!("higher_half  : 0x{:x}", kfs::boot::higher_half as *const () as usize);

    // arch::x86::gdt::init();
    // arch::x86::idt::init();

    init_memory(info.mem_upper as usize, info.mem_lower as usize);

    //     let mut value: u32;
    //     unsafe {

    //     asm!("mov {}, cr0", out(reg) value);
    //     }
    //     printkln!("REGISTER: {:b}", value);
    //     value |= 1 << 16;
    //     unsafe {

    //     asm!("mov cr0, {}", in(reg) value);
    //     }

    // let addr  = vmm::mmap(None,  PAGE_SIZE, vmm::Permissions::Read, vmm::Access::User);
    // let addr = match addr {
    //     Ok(addr) => {printkln!("return addr: 0x{:x}", addr);  addr},
    //     Err(err) => {printkln!("{:?}", err); 0}
    // };
    // let addr = 0x1000;
    // let ptr = addr as *mut u8;       // interpret as pointer to i32

    // unsafe {
    //     // ⚠️ undefined behavior if the address is invalid/unmapped/unaligned
    //     // *ptr = 42;
    //     let value = *ptr;
    //     printkln!("Value at {:X} = {}", addr, value);

    // }

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

// #[cfg(test)]
// #[unsafe(no_mangle)]
// pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
//     use kfs::{arch, qemu, vmm};

//     arch::x86::gdt::init();
//     arch::x86::idt::init();

//     vmm::init_memory(info.mem_upper as usize, info.mem_lower as usize);

//     test_main();
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
