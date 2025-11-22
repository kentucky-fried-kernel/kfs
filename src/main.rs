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

fn set_bit(num: &mut usize, bit_position: u8) {
    *num |= 1 << bit_position
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub unsafe extern "C" fn trampolin() {
    let x: usize = 10;
    printk!("0x{:x}\n", (trampolin as *const usize) as usize);
    kernel_main();
}

// #[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use core::arch::asm;

    use kfs::{arch, printk, ps2, shell, terminal};
    if let Err(e) = ps2::init() {
        panic!("could not initialize PS/2: {}", e);
    }
    arch::x86::gdt::init();

    let mut dir_table: DirectoryTable = DirectoryTable { entries: [0; 1024] };
    let mut pages_table: DirectoryTable = DirectoryTable { entries: [0; 1024] };

    dir_table.entries[0] = (&mut pages_table as *mut DirectoryTable) as usize;

    // make 0xc0000000..(0xc0000000 + 4mb) point to kernel as well
    let kernel_pde_index = (0xC0000000 >> 22) & 0x3FF;
    dir_table.entries[kernel_pde_index] = (&mut pages_table as *mut DirectoryTable) as usize;

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
    for i in (0..32).rev() {
        printk!("{}", (dir_table.entries[0] >> i) & 1);
    }
    printk!("\n");
    printk!("{:x}\n", cr3_value);
    printk!("{:x}\n", dir_table.entries[0]);
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
