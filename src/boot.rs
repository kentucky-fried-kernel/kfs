use core::{fmt::Display, ops::BitOr};

pub const STACK_SIZE: usize = 2 << 20;

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

#[allow(unused)]
#[derive(Debug)]
pub struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    syms: [u32; 3],
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u32,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,
    framebuffer_addr: u32,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    color_info: [u8; 5],
}

impl Display for MultibootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "MultibootInfo {{")?;
        writeln!(f, "  flags: 0x{:b}", self.flags)?;
        writeln!(f, "  mem_lower: {} KB", self.mem_lower)?;
        writeln!(f, "  mem_upper: {} KB", self.mem_upper)?;
        writeln!(f, "  boot_device: 0x{:x}", self.boot_device)?;
        writeln!(f, "  cmdline: 0x{:x}", self.cmdline)?;
        writeln!(f, "  mods_count: {}", self.mods_count)?;
        writeln!(f, "  mods_addr: 0x{:x}", self.mods_addr)?;
        writeln!(f, "  mmap_length: {}", self.mmap_length)?;
        writeln!(f, "  mmap_addr: 0x{:x}", self.mmap_addr)?;
        writeln!(f, "  boot_loader_name: 0x{:x}", self.boot_loader_name)?;
        writeln!(
            f,
            "  framebuffer: {}x{}x{} @ 0x{:x}",
            self.framebuffer_width, self.framebuffer_height, self.framebuffer_bpp, self.framebuffer_addr
        )?;
        write!(f, "}}")
    }
}

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
static INITIAL_PAGE_DIR: [usize; 1024] = {
    let mut dir = [0usize; 1024];

    dir[0] = 0b10000011;

    dir[768] = (0 << 22) | 0b10000011;
    dir[769] = (1 << 22) | 0b10000011;
    dir[770] = (2 << 22) | 0b10000011;
    dir[771] = (3 << 22) | 0b10000011;

    dir
};

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
        "push ebx",
        "push eax",
        "xor ebp, ebp",
        "call kmain",
        "halt:",
        "hlt",
        "jmp halt",
        STACK_SIZE = const STACK_SIZE
    )
}

/// # Safety
/// This function marks the entrypoint of the kernel executable.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        "mov ecx, offset INITIAL_PAGE_DIR - 0xC0000000",
        "mov cr3, ecx",
        "mov ecx, cr4",
        "or ecx, 0x10",
        "mov cr4, ecx",
        "mov ecx, cr0",
        "or ecx, 0x80000000",
        "mov cr0, ecx",
        "jmp higher_half",
    )
}
