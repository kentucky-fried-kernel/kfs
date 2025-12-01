use core::{fmt::Display, ops::BitOr};

pub const STACK_SIZE: usize = 2 << 20;
pub const KERNEL_BASE: usize = 0xC0000000;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss")]
pub static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[allow(unused)]
#[repr(align(4096))]
#[allow(static_mut_refs)]
pub struct Stack([u8; STACK_SIZE]);

impl Stack {
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}

#[allow(unused)]
#[repr(align(4))]
struct MultibootHeader {
    magic: usize,
    flags: usize,
    checksum: usize,
}

#[repr(usize)]
pub enum MultibootFlag {
    Mem = 1 << 0,
    BootDevice = 1 << 1,
    Cmdline = 1 << 2,
    Mods = 1 << 3,
    Syms = 1 << 4 | 1 << 5,
    Mmap = 1 << 6,
    Drives = 1 << 7,
    ConfigTable = 1 << 8,
    Bootloader = 1 << 9,
    Apm = 1 << 10,
    Vbe = 1 << 11,
    Framebuffer = 1 << 12,
}

impl const BitOr for MultibootFlag {
    type Output = usize;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as usize | rhs as usize
    }
}

const MULTIBOOT_FLAGS: usize = MultibootFlag::Mem | MultibootFlag::BootDevice;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot")]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: 0x1badb002,
    flags: MULTIBOOT_FLAGS,
    checksum: (0usize.wrapping_sub(0x1badb002 + (MULTIBOOT_FLAGS))),
};

#[derive(Debug)]
#[repr(C, align(4))]
pub struct MultibootInfo {
    pub flags: u32,
    pub mem_lower: u32,
    pub mem_upper: u32,
    pub boot_device: u32,
    pub cmdline: u32,
    pub mods_count: u32,
    pub mods_addr: u32,
    pub syms: [u32; 4],
    pub mmap_length: u32,
    pub mmap_addr: u32,
    pub drives_length: u32,
    pub drives_addr: u32,
    pub config_table: u32,
    pub boot_loader_name: u32,
    pub apm_table: u32,
    pub vbe_control_info: u32,
    pub vbe_mode_info: u32,
    pub vbe_mode: u32,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,
    pub framebuffer_addr: u32,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
    pub framebuffer_type: u8,
    pub color_info: [u8; 5],
}

impl Display for MultibootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "MultibootInfo {{")?;
        writeln!(f, "  flags: 0b{:b}", self.flags)?;
        writeln!(f, "  mem_lower: {} KB", self.mem_lower)?;
        writeln!(f, "  mem_upper: {} KB", self.mem_upper)?;
        writeln!(f, "  boot_device: 0x{:x}", self.boot_device)?;
        writeln!(f, "  mmap_addr, 0x{:x}", self.mmap_addr)?;
        writeln!(f, "  mmap_length, 0x{:x}", self.mmap_length)?;
        write!(f, "}}")
    }
}

#[repr(C, align(1))]
pub struct MultibootMmapEntry {
    pub size: u32,
    pub addr: u64,
    pub len: u64,
    pub ty: u32,
}

#[repr(C)]
pub struct MemoryMap {
    htype: u32,
    size: u32,
    entry_size: u32,
    versions: u32,
    entries: [MultibootMmapEntry; 0],
}

/// # Safety
/// This function is used as a marker _start can jump to after initializing
/// paging. It is **not** meant to be called, and is marked as unsafe
/// because it uses `naked_asm`.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn higher_half() {
    core::arch::naked_asm!(
        "mov esp, offset STACK + {STACK_SIZE}",
        "add ebx, {KERNEL_BASE}",
        "push ebx",
        "push eax",
        "xor ebp, ebp",
        "call kmain",
        "halt:",
        "hlt",
        "jmp halt",
        STACK_SIZE = const STACK_SIZE,
        KERNEL_BASE = const KERNEL_BASE
    )
}

/// # Safety
/// This function marks the entrypoint of the kernel executable.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        "mov ecx, offset KERNEL_PAGE_DIRECTORY_TABLE - {KERNEL_BASE}",
        "mov cr3, ecx",
        "mov ecx, cr4",
        "or ecx, 0x10",
        "mov cr4, ecx",
        "mov ecx, cr0",
        "or ecx, 0x80000000",
        "mov cr0, ecx",
        "jmp higher_half",
        KERNEL_BASE = const KERNEL_BASE
    )
}
