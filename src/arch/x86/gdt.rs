use core::arch::asm;

use crate::boot::{KERNEL_BASE, STACK, STACK_SIZE};

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

const GDT_SIZE: usize = 8;
static mut GDT: GdtTable = GdtTable { entries: [0u64; GDT_SIZE] };

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

#[unsafe(no_mangle)]
static mut GDTR: Gdtr = Gdtr {
    limit: ((GDT_SIZE * 8) - 1) as u16,
    base: 0,
};

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

    #[allow(static_mut_refs)]
    let tss = unsafe { &mut TSS };
    #[allow(static_mut_refs)]
    let stack = unsafe { &STACK as *const _ as u32 };
    tss.esp0 = stack + STACK_SIZE as u32 - 4;
    tss.ss0 = 0x10;
    tss.iopb = 104;
    gdt.entries[7] = create_gdt_descriptor(0x0089, (size_of::<TaskSwitchSegment>() - 1) as u32, unsafe { tss as *const _ as u32 });

    // SAFETY:
    // We know this is safe since this module is the only one that can access GDTR.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    unsafe {
        GDTR.base = &raw const GDT as u32;
    }

    // SAFETY:
    // We make sure that GDTR is properly initialized before loading it.
    unsafe { flush_gdt_registers() };

    unsafe {
        asm!("ltr ax", in("ax") 0x38u16);
    }
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

const SEGMENT_SELECTOR_KERNEL: u32 = 0x10;

static mut TSS: TaskSwitchSegment = TaskSwitchSegment {
    prev_tss: 0,
    esp0: 0,
    ss0: SEGMENT_SELECTOR_KERNEL,
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
};
