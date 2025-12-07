use core::ptr::write_volatile;

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

const GDT_SIZE: usize = 7;
const GDT_ADDRESS: *mut u64 = 0x0000_0800 as *mut u64;

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

    for (i, entry) in gdt.iter().enumerate() {
        // SAFETY:
        // When `gdt::init` is called, `GDT_ADDRESS` is guaranteed to be a valid,
        // writable address.
        let ptr = unsafe { GDT_ADDRESS.add(i) };
        // SAFETY:
        // We write to GDT_ADDRESS, which we know is valid.
        unsafe { write_volatile(ptr, *entry) };
    }

    // SAFETY:
    // `flush_gdt_registers()` needs to be unsafe due to the naked assembly.
    unsafe { flush_gdt_registers() };
}
