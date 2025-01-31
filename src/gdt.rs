use core::{arch::asm, ptr::write_volatile};

use crate::print::u64_to_base;

fn create_gdt_descriptor(flags: u16, limit: u32, base: u32) -> u64 {
    let mut descriptor: u64 = 0;

    descriptor = (limit as u64) & 0x000F0000;
    descriptor |= ((flags as u64) << 8) & 0x00F0FF00;
    descriptor |= ((base as u64) >> 16) & 0x000000FF;
    descriptor |= (base as u64) & 0xFF000000;
    descriptor <<= 32;
    descriptor |= (base as u64) << 16;
    descriptor |= (limit as u64) & 0x0000FFFF;

    descriptor
}

#[no_mangle]
static GDT_SIZE: usize = 7;
static GDT_LIMIT: usize = GDT_SIZE - 1;
#[no_mangle]
static mut GDT: [u64; GDT_SIZE] = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

const GDT_ADDRESS: *mut u64 = 0x00000800 as *mut u64;

extern "C" {
    fn flush_gdt_registers() -> u32;
}

#[allow(named_asm_labels)]
pub fn set_gdt() {
    unsafe {
        GDT[1] = create_gdt_descriptor(0xC09A, 0xFFFFF, 0x0);
        GDT[2] = create_gdt_descriptor(0xC092, 0xFFFFF, 0x0);
        GDT[3] = GDT[2];
        GDT[4] = create_gdt_descriptor(0xC0FA, 0xFFFFF, 0x0);
        GDT[5] = create_gdt_descriptor(0xC0F2, 0xFFFFF, 0x0);
        GDT[6] = GDT[5];

        for (i, entry) in GDT.iter().enumerate() {
            write_volatile(GDT_ADDRESS.add(i), *entry);
        }

        flush_gdt_registers();
    }
}
