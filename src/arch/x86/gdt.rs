use bitmap::{bitmap, traits::*};
use core::ptr::write_volatile;

#[bitmap]
struct GdtDescriptor {
    base_high: u8,
    flags: u4,
    limit_high: u4,
    access_byte: u8,
    base_mid: u8,
    base_low: u16,
    limit_low: u16,
}

fn create_gdt_descriptor(flags: u16, limit: u32, base: u32) -> u64 {
    let mut descriptor = GdtDescriptor::new(0);

    descriptor
        .set_limit_high(limit.get_bits(16..20) as u8)
        .set_flags(flags.get_bits(12..16) as u8)
        .set_access_byte(flags.get_bits(0..8) as u8)
        .set_base_high(base.get_bits(24..32) as u8)
        .set_base_mid(base.get_bits(16..24) as u8)
        .set_base_low(base.get_bits(0..16) as u16)
        .set_limit_low(limit.get_bits(0..16) as u16);

    descriptor.into()
}

const GDT_SIZE: usize = 7;
const GDT_ADDRESS: *mut u64 = 0x00000800 as *mut u64;

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

#[unsafe(no_mangle)]
static GDTR: Gdtr = Gdtr { limit: 0x37, base: 0x800 };

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn flush_gdt_registers() {
    core::arch::naked_asm!(
        "mov eax, offset GDTR",
        "lgdt [eax]",
        "mov eax, cr0",
        "or eax, 1",
        "mov cr0, eax",
        "jmp 0x08, offset flush",
        "flush:",
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        "ret",
    );
}

pub fn init() {
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
