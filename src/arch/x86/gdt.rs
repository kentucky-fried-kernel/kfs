pub const KERNEL_CODE_OFFSET: usize = 0x08;

fn create_gdt_descriptor(flags: u16, limit: u32, base: u32) -> u64 {
    let mut descriptor: u64;

    descriptor = u64::from(limit) & 0x000F_0000;
    descriptor |= (u64::from(flags) << 8) & 0x00F0_FF00;
    descriptor |= (u64::from(base) >> 16) & 0x0000_00FF;
    descriptor |= u64::from(base) & 0xFF00_0000;
    descriptor <<= 32;
    descriptor |= u64::from(base) << 16;
    descriptor |= u64::from(limit) & 0x0000_FFFF;

    descriptor
}

struct GdtTable {
    entries: [u64; GDT_SIZE],
}

const GDT_SIZE: usize = 7;
static mut GDT: GdtTable = GdtTable { entries: [0u64; GDT_SIZE] };

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

#[unsafe(no_mangle)]
static mut GDTR: Gdtr = Gdtr { limit: 0x37, base: 0 };

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
    // SAFETY:
    // We know this is safe since this module is the only one that can access GDT.
    #[allow(static_mut_refs)]
    let gdt = unsafe { &mut GDT };

    gdt.entries[1] = create_gdt_descriptor(0xC09A, 0xFFFFF, 0x0);
    gdt.entries[2] = create_gdt_descriptor(0xC092, 0xFFFFF, 0x0);
    gdt.entries[3] = gdt.entries[2];
    gdt.entries[4] = create_gdt_descriptor(0xC0FA, 0xFFFFF, 0x0);
    gdt.entries[5] = create_gdt_descriptor(0xC0F2, 0xFFFFF, 0x0);
    gdt.entries[6] = gdt.entries[5];

    // SAFETY:
    // We know this is safe since this module is the only one that can access GDTR.
    unsafe { GDTR.base = &raw const gdt as u32 };

    // SAFETY:
    // We make sure that GDTR is properly initialized before loading it.
    unsafe { flush_gdt_registers() };
}
