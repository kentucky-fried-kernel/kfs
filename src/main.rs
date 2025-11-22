#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::asm;

use kfs::{printk, qemu::exit};

mod panic;

// type DirectoryEntry = usize;
// #[unsafe(link_section = "page_table")]
// static DIRECTORY_ENTRIES: [DirectoryEntry; 1024] = [0; 1024];

#[repr(align(0x1000))]
struct DirectoryTable {
    entries: [usize; 1024],
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
fn set_bit(num: &mut usize, bit_position: u8) {
    *num |= 1 << bit_position
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot_page_dir")]
static mut dir_table: DirectoryTable = DirectoryTable { entries: [0; 1024] };
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot_page_entries")]
static mut pages_table: DirectoryTable = DirectoryTable { entries: [0; 1024] };

#[unsafe(link_section = ".boot_page_entries")]
static print: &str = "hello";
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
#[allow(static_mut_refs)]
pub unsafe extern "C" fn trampolin() {

    
    
    asm!("hlt");
    dir_table.entries[0] = (&mut pages_table as *mut DirectoryTable) as usize;
    // make 0xc0000000..(0xc0000000 + 4mb) point to kernel as well
    let kernel_pde_index = (0xC0000000 >> 22) & 0x3FF;
    dir_table.entries[kernel_pde_index] = (&mut pages_table as *mut DirectoryTable) as usize;

    let dir_entry = &mut dir_table.entries[kernel_pde_index];
    set_bit(dir_entry, 1);
    set_bit(dir_entry, 0);

    let dir_entry = &mut dir_table.entries[0];
    set_bit(dir_entry, 1);
    set_bit(dir_entry, 0);

    for i in 0..1024 {
        let page_entry = &mut pages_table.entries[i];
        *page_entry = i * 0x1000;
        set_bit(page_entry, 0);
        set_bit(page_entry, 1);
    }

    let cr3_value = (&dir_table as *const DirectoryTable) as usize;

    unsafe {
        asm!(
            "mov cr3, {0}",
            in(reg) cr3_value,
            options(nostack, preserves_flags)
        );

        let mut cr0_value: u32;
        asm!(
            "mov {0}, cr0",
            out(reg) cr0_value,
            options(nostack, preserves_flags)
        );

        cr0_value |= 0x80000000;
        asm!(
            "mov cr0, {0}",  // move the new value back into cr0
            in(reg) cr0_value, // input the modified value
            options(nostack, preserves_flags)
        );
    }
    // unsafe {
    //     asm!("mov esp, offset STACK1 + {stack_size}",
    //     "mov ebp, offset STACK1",
    //     "push eax",
    //     "push ebx",
    //     stack_size = const STACK_SIZE
    //     )
    // }
    kernel_main();
}
// const STACK_SIZE: usize = 1 << 21;

// #[used]
// #[unsafe(no_mangle)]
// #[unsafe(link_section = ".bss")]
// pub static mut STACK1: [u8; STACK_SIZE] = [0; STACK_SIZE];

// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use core::arch::asm;
    use kfs::{arch, shell, terminal};

    // if let Err(e) = ps2::init() {
    //     panic!("could not initialize PS/2: {}", e);
    // }
    // let f = ((arch::x86::gdt::init as *const () as usize) - 0xC0000000);

    //     let f: *const () = arch::x86::gdt::init as *const ();

    // let base: usize = 0xC0000000;
    // let function_address: usize = f as usize;
    // let relative_address = function_address - base;

    // let f_fn: unsafe fn() = relative_address as *const () as *const ();

    // unsafe {
    //     f_fn();
    // }


    // arch::x86::gdt::init();
    // unsafe {
    //     *(0xB8000 as * mut u16) = 0x4f << 8 | (b'A' as u16);
    //     asm!("hlt");
    // }


    // printk!("|{:?}|", dir_table.entries[0]);
    // printk!("|{:?}|", &pages_table as *const _);


    // arch::x86::set_idt();
    #[allow(static_mut_refs)]
    shell::launch(unsafe { &mut terminal::SCREEN });
}

// #[cfg(test)]
// #[unsafe(no_mangle)]
// pub extern "C" fn kernel_main() {
//     use kfs::qemu;
//     test_main();
//     unsafe { qemu::exit(qemu::ExitCode::Success) };
// }
