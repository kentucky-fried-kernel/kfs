use crate::serial_println;

#[derive(Copy, Clone)]
struct GdtEntry(u64);

impl GdtEntry {
    pub const fn disabled() -> Self {
        Self(0)
    }

    pub fn new(flags: u16, limit: u32, base: u32) -> Self {
        let mut entry: u64;

        entry = u64::from(limit) & 0x000F_0000;
        entry |= (u64::from(flags) << 8) & 0x00F0_FF00;
        entry |= (u64::from(base) >> 16) & 0x0000_00FF;
        entry |= u64::from(base) & 0xFF00_0000;
        entry <<= 32;
        entry |= u64::from(base) << 16;
        entry |= u64::from(limit) & 0x0000_FFFF;

        Self(entry)
    }
}

type Gdt = GlobalDescriptorTable;
#[derive(Copy, Clone)]
struct GlobalDescriptorTable {
    entries: [GdtEntry; GDT_SIZE],
}

type Gdtr = GlobalDescriptorTableRegister;
#[repr(C, packed)]
struct GlobalDescriptorTableRegister {
    limit: u16,
    base: u32,
}

const GDT_SIZE: usize = 7;
static mut GDT: Gdt = Gdt {
    entries: [GdtEntry::disabled(); GDT_SIZE],
};

#[unsafe(no_mangle)]
static mut GDTR: Gdtr = Gdtr { limit: 0x37, base: 0 };
pub const SEGMENT_SELECTOR_KERNEL_CODE: usize = 1;
pub const SEGMENT_SELECTOR_KERNEL_DATA: usize = 2;
pub const SEGMENT_SELECTOR_USER_CODE: usize = 3;
pub const SEGMENT_SELECTOR_USER_DATA: usize = 4;

pub fn init() {
    // SAFETY:
    // We know this is safe since this module is the only one that can access GDT.
    #[allow(static_mut_refs)]
    let gdt = unsafe { &mut GDT };

    gdt.entries[SEGMENT_SELECTOR_KERNEL_CODE] = GdtEntry::new(0xC09A, 0xFFFFF, 0x0);
    gdt.entries[SEGMENT_SELECTOR_KERNEL_DATA] = GdtEntry::new(0xC092, 0xFFFFF, 0x0);
    gdt.entries[SEGMENT_SELECTOR_USER_CODE] = GdtEntry::new(0xC0FA, 0xFFFFF, 0x0);
    gdt.entries[SEGMENT_SELECTOR_USER_DATA] = GdtEntry::new(0xC0F2, 0xFFFFF, 0x0);

    // SAFETY:
    // We know this is safe since this module is the only one that can access GDTR.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        GDTR.base = &raw const GDT as u32;
    }

    // SAFETY:
    // We make sure that GDTR is properly initialized before loading it.
    unsafe { gdt_set() };

    // SAFETY:
    // We make sure that the values passed are valid segment selectors.
    unsafe {
        // Reload segment registers "invisible" parts by setting them again.
        segment_registers_reload();
    }
}

#[unsafe(naked)]
unsafe extern "C" fn segment_registers_reload() {
    core::arch::naked_asm!(
        "jmp {SELECTOR_CODE}, offset flush",
        "flush:",
        "mov ax, {SELECTOR_DATA}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        "ret",
        SELECTOR_CODE = const SEGMENT_SELECTOR_KERNEL_CODE << 3,
        SELECTOR_DATA = const SEGMENT_SELECTOR_KERNEL_DATA << 3,

    );
}

#[unsafe(naked)]
unsafe extern "C" fn gdt_set() {
    core::arch::naked_asm!("mov eax, offset GDTR", "lgdt [eax]", "mov eax, cr0", "or eax, 1", "mov cr0, eax", "ret",);
}
