use core::arch::asm;

use crate::{
    arch::x86::kernel_mutex::KernelMutex,
    boot::{STACK, STACK_SIZE},
};

#[derive(Copy, Clone)]
#[allow(unused)]
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

const GDT_SIZE: usize = 6;
static mut GDT: Gdt = Gdt {
    entries: [GdtEntry::disabled(); GDT_SIZE],
};

#[unsafe(no_mangle)]
static mut GDTR: Gdtr = Gdtr {
    limit: (GDT_SIZE * 8 - 1) as u16,
    base: 0,
};
pub const SEGMENT_SELECTOR_KERNEL_CODE: usize = 1;
pub const SEGMENT_SELECTOR_KERNEL_DATA: usize = 2;
pub const SEGMENT_SELECTOR_USER_CODE: usize = 3;
pub const SEGMENT_SELECTOR_USER_DATA: usize = 4;
pub const SEGMENT_SELECTOR_TSS: usize = 5;

pub const SEGMENT_MODE_KERNEL: u32 = 0b00;
pub const SEGMENT_MODE_USER: u32 = 0b11;

#[allow(clippy::missing_panics_doc)]
pub fn init() {
    // SAFETY:
    // We know this is safe since this module is the only one that can access GDT.
    #[allow(static_mut_refs)]
    let gdt = unsafe { &mut GDT };

    gdt.entries[SEGMENT_SELECTOR_KERNEL_CODE] = GdtEntry::new(0xC09A, 0xFFFFF, 0x0);
    gdt.entries[SEGMENT_SELECTOR_KERNEL_DATA] = GdtEntry::new(0xC092, 0xFFFFF, 0x0);
    gdt.entries[SEGMENT_SELECTOR_USER_CODE] = GdtEntry::new(0xC0FA, 0xFFFFF, 0x0);
    gdt.entries[SEGMENT_SELECTOR_USER_DATA] = GdtEntry::new(0xC0F2, 0xFFFFF, 0x0);

    #[allow(clippy::expect_used)]
    let mut tss = TSS.lock().expect("could not lock TSS on init");
    let tss_vaddr = &raw const *tss as u32;

    let stack_vaddr = &raw const STACK as u32;
    tss.esp0 = stack_vaddr + STACK_SIZE as u32 - 4;
    tss.ss0 = (SEGMENT_SELECTOR_KERNEL_DATA << 3) as u32; // we don't need to set permission
    // because it defaults to 00 (kernel)
    // which we need.
    tss.iopb = size_of::<TaskSwitchSegment>() as u16;
    gdt.entries[SEGMENT_SELECTOR_TSS] = GdtEntry::new(0x0089, (size_of::<TaskSwitchSegment>() - 1) as u32, tss_vaddr);

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
    // This is safe because we are still in the kernel context to which
    // the segment selectors get realoaded to.
    unsafe {
        // Reload segment registers "invisible" parts by setting them again.
        segment_registers_reload();
    }

    // SAFETY:
    // This is safe because we setup TSS before
    unsafe {
        asm!("ltr ax", in("ax") SEGMENT_SELECTOR_TSS << 3);
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
        SELECTOR_CODE = const SEGMENT_SELECTOR_KERNEL_CODE << 3, // the index gets shifted by 3
                                                                 // because the bits 15..3 in the
                                                                 // segment selector are used for
                                                                 // the index of the gdt.
        SELECTOR_DATA = const SEGMENT_SELECTOR_KERNEL_DATA << 3,

    );
}

#[unsafe(naked)]
unsafe extern "C" fn gdt_set() {
    core::arch::naked_asm!("mov eax, offset GDTR", "lgdt [eax]", "mov eax, cr0", "or eax, 1", "mov cr0, eax", "ret",);
}

#[repr(C)]
pub struct TaskSwitchSegment {
    pub prev_tss: u32,
    pub esp0: u32,
    pub ss0: u32,
    pub esp1: u32,
    pub ss1: u32,
    pub esp2: u32,
    pub ss2: u32,
    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub es: u32,
    pub cs: u32,
    pub ss: u32,
    pub ds: u32,
    pub fs: u32,
    pub gs: u32,
    pub ldt: u32,
    pub trap: u16,
    pub iopb: u16, // ← set to size_of::<Tss>() = 104
}

static TSS: KernelMutex<TaskSwitchSegment> = KernelMutex::new(TaskSwitchSegment {
    prev_tss: 0,
    esp0: 0,
    ss0: (SEGMENT_SELECTOR_KERNEL_CODE << 3) as u32,
    esp1: 0,
    ss1: 0,
    esp2: 0,
    ss2: 0,
    cr3: 0,
    eip: 0,
    eflags: 0,
    eax: 0,
    ecx: 0,
    edx: 0,
    ebx: 0,
    esp: 0,
    ebp: 0,
    esi: 0,
    edi: 0,
    es: 0,
    cs: 0,
    ss: 0,
    ds: 0,
    fs: 0,
    gs: 0,
    ldt: 0,
    trap: 0,
    iopb: size_of::<TaskSwitchSegment>() as u16,
});
