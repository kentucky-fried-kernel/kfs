#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kfs::boot::{KERNEL_BASE, KERNEL_PAGE_DIR, KERNEL_PAGE_ENTRIES, MultibootInfo};

mod panic;

#[derive(Copy, Clone)]
struct Range {
    pub start: u32,
    pub size: u32
}

impl Range {
    fn new() -> Self {
        Range {
            start: 0,
            size: 0,
        }
    }
}

unsafe extern "C" {
    #[link_name = "_kernel_end"]
    static KERNEL_END: u8;
}


#[cfg(not(test))]
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

    arch::x86::gdt::init();
    arch::x86::idt::init();

    let kernel_end = unsafe {&KERNEL_END as *const u8} as usize;

    printkln!("addressssss 0x{:x}", kernel_end);

    let kernel_pages_needed = ((kernel_end + kernel_end % 0x1000) - KERNEL_BASE) / 0x1000;

    for i in 0..kernel_pages_needed {
        let dir_index = i / 1024;
        let page_index = i % 1024;
        unsafe {
            KERNEL_PAGE_ENTRIES[dir_index][page_index] = i << 12 | 0b11;
        }
    }

    let mut kernel_page_entries_physical_address = &raw const KERNEL_PAGE_ENTRIES as usize;
    kernel_page_entries_physical_address -= KERNEL_BASE;


    for i in 0..=(kernel_pages_needed / 1024) {
        unsafe {
            KERNEL_PAGE_DIR[768 + i] = (kernel_page_entries_physical_address / 0x1000) + i << 12 | 0b11;
        }
    }

    // let available_ranges: [Range; 64] = [Range::new(); 64];
    // let mut i = 0;
    // let mut arI = 0;

    // printkln!("start table");
    // loop {
    //     use kfs::boot::MultibootMmapEntry;

    //     unsafe {
    //         let entry: *const MultibootMmapEntry = (info.mmap_addr + i) as *const MultibootMmapEntry;
    //         printkln!("addr: 0x{:09X} | len : 0x{:08X} | type : {:x}", (*entry).addr , (*entry).len , (*entry).ty);
    //         if (*entry).addr >= 0x100000 {
    //             let last_end = match arI {
    //                 0 => 0x100000,
    //                 _ => available_ranges[arI -1].start + available_ranges[arI -1].size
    //             };

    //             if (*entry).addr <= last_end {

    //             }
    //         }
    //         i += (*entry).size + 4;
    //         if i >= info.mmap_length {
    //             break;
    //         }
    //     }
    // }

    vmm::init_memory(info.mem_upper as usize, info.mem_lower as usize);

    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{arch, qemu, vmm};

    arch::x86::gdt::init();
    arch::x86::idt::init();

    vmm::init_memory(info.mem_upper as usize, info.mem_lower as usize);

    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
