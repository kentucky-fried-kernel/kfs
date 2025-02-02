use core::ptr::write_volatile;

fn create_gdt_descriptor(flags: u16, limit: u32, base: u32) -> u64 {
    let mut descriptor: u64;

    descriptor = (limit as u64) & 0x000F0000;
    descriptor |= ((flags as u64) << 8) & 0x00F0FF00;
    descriptor |= ((base as u64) >> 16) & 0x000000FF;
    descriptor |= (base as u64) & 0xFF000000;
    descriptor <<= 32;
    descriptor |= (base as u64) << 16;
    descriptor |= (limit as u64) & 0x0000FFFF;

    descriptor
}

const GDT_SIZE: usize = 7;
const GDT_ADDRESS: *mut u64 = 0x00000800 as *mut u64;

unsafe extern "C" {
    unsafe fn flush_gdt_registers() -> u32;
}

pub fn set_gdt() {
    let mut gdt: [u64; GDT_SIZE] = [0u64; GDT_SIZE];
    gdt[1] = create_gdt_descriptor(0xC09A, 0xFFFFF, 0x0);
    gdt[2] = create_gdt_descriptor(0xC092, 0xFFFFF, 0x0);
    gdt[3] = gdt[2];
    gdt[4] = create_gdt_descriptor(0xC0FA, 0xFFFFF, 0x0);
    gdt[5] = create_gdt_descriptor(0xC0F2, 0xFFFFF, 0x0);
    gdt[6] = gdt[5];

    unsafe {
        for (i, entry) in gdt.iter().enumerate() {
            write_volatile(GDT_ADDRESS.add(i), *entry);
        }

        flush_gdt_registers();
    }
}
