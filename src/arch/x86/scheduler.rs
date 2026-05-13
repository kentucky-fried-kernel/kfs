#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
use core::arch::naked_asm;

use alloc::vec::Vec;

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        interrupts::irq,
        kernel_mutex::KernelMutex,
        vmm::process::{Process, Scheduler},
    },
    serial_println,
};

#[unsafe(naked)]
#[allow(unused)]
extern "C" fn program_exit() {
    naked_asm!("aaa:", "mov eax, 60", "int 0x80", "jmp aaa");
}

#[unsafe(naked)]
extern "C" fn program_write_in_memory() {
    naked_asm!(
        // both processes start by writing the same initial value
        "mov dword ptr [0x2000], 0x42",
        // fork
        "mov eax, 57",
        "int 0x80",
        // after this: eax = 0 in child, eax = child_pid in parent

        // branch on eax
        "test eax, eax",
        "jz child",
        // ----- parent path -----
        // overwrite with 0xAAAA - this should NOT affect the child
        "mov dword ptr [0x2000], 0xAAAA",
        "jmp done",
        "child:",
        // ----- child path -----
        // overwrite with 0xBBBB - this should NOT affect the parent
        "mov dword ptr [0x2000], 0xBBBB",
        "done:",
        "mov ebx, [0x2000]",
        "mov eax, 42",
        "int 0x80",
        // both processes read their own [0x2000] into ebx and exit
        "mov eax, 60",
        "int 0x80",
    );
}
//
// #[unsafe(naked)]
// extern "C" fn program_ipc_test_sender() {
//     // naked_asm!()
// }
#[unsafe(naked)]
extern "C" fn program_ipc_test() {
    naked_asm!(
        // --- socket_create() -> eax = fd ---
        "mov eax, 5",
        "int 0x80",
        // save fd in esi (callee-saved-ish for our purposes; nothing clobbers
        // it until we use it again).
        "mov esi, eax",
        //
        // --- fork() ---
        "mov eax, 57",
        "int 0x80",
        //
        //
        // branch on eax: 0 = child, nonzero = parent
        "test eax, eax",
        "jz child",
        "mov edi, eax",
        // edi is now the child_id

        // Store pid at [0x2000] so we have a stable address to pass as buf.
        "mov dword ptr [0x2000], edi",
        // socket_write(fd=esi, buf=0x2000, len=4)
        "mov eax, 8",
        "mov ebx, esi",
        "mov ecx, 0x2000",
        "mov edx, 4",
        "int 0x80",
        "mov ebx, eax",
        "mov eax, 42",
        "int 0x80",
        // exit(0). ebx = 0 so the parent's exit log is unambiguous.
        "mov eax, 60",
        "mov ebx, 0",
        "int 0x80",
        // ===== CHILD =====
        "child:",
        // Busy-loop on socket_read until it returns > 0.
        // (No blocking read yet, so we spin.)
        "retry:",
        "mov eax, 7",
        "mov ebx, esi",
        "mov ecx, 0x2000",
        "mov edx, 4",
        "int 0x80",
        // eax = bytes read. If 0, retry.
        "test eax, eax",
        "jz retry",
        // Load the received value into ebx and exit, so sys_exit prints it.
        "mov ebx, [0x2000]",
        //
        "mov eax, 42",
        "int 0x80",
        //
        "mov eax, 60",
        "int 0x80",
    );
}

// #[unsafe(naked)]
// extern "C" fn program() {
//     naked_asm!(
//         // write 0x42 to address 0x2000
//         "mov dword ptr [0x2000], 0x42",
//         // fork
//         "mov eax, 57",
//         "int 0x80",
//         // eax is now 0 in child, child_pid in parent
//
//         // both parent and child now read from 0x2000
//         // they should each see 0x42 because the page was copied
//         "mov ebx, [0x2000]",
//         // exit with the value we read as the exit code
//         "mov eax, 60",
//         "int 0x80",
//     );
// }
// #[unsafe(naked)]
// extern "C" fn program() {
//     naked_asm!("aaaa:", "mov eax, 57", "int 0x80", "mov ebx, eax", "mov eax, 60", "int 0x80",
// "jmp aaaa"); }

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Permissions {
    Read = 0,
    ReadWrite = 1,
}

pub struct Segment {
    pub offset: Option<usize>,
    pub vaddr: usize,
    pub size: usize,
    pub permissions: Permissions,
}

pub struct Binary {
    pub segments: Vec<Segment>,
    pub entry: usize,
    pub stack: usize,
}

impl Binary {
    #[must_use]
    pub fn new(entry: usize, stack: usize) -> Self {
        Self {
            segments: Vec::new(),
            entry,
            stack,
        }
    }
}

#[allow(unused)]
static ONE: u32 = 1;

#[allow(unused)]
static TWO: u32 = 2;

#[allow(clippy::missing_panics_doc)]
pub fn init() {
    let mut bin = Binary::new(0x1000, 0x3000);

    // Code page: ipc test program.
    bin.segments.push(Segment {
        offset: Some(program_ipc_test as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    // Data page used as the IPC payload buffer at 0x2000.
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x2000,
        size: 0x1000,
        permissions: Permissions::ReadWrite,
    });

    // Stack page.
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x3000,
        size: 0x1000,
        permissions: Permissions::ReadWrite,
    });
    let p = Process::new(&bin, super::vmm::process::Parent::Root);
    SCHEDULER.lock().unwrap().spawn(p);

    irq::install_handler(0, timer);
    irq::clear_mask(0);
}

pub static SCHEDULER: KernelMutex<Scheduler> = KernelMutex::new(Scheduler::new());

#[allow(clippy::missing_panics_doc)]
pub extern "C" fn timer(regs: &mut InterruptRegisters) {
    serial_println!("timer");
    serial_println!("{:x}", regs.eax);
    let mut scheduler = SCHEDULER.lock().expect("timer | failed to lock SCHEDULER");
    // first save the registers if there was a running process
    if let Some(p) = scheduler.current() {
        p.saved_registers = *regs;
    }

    // find the next process
    let next = scheduler.schedule().expect("could not lock SCHEDULER");

    next.state = crate::arch::x86::vmm::process::ProcessState::Running;
    next.addressspace.load();
    *regs = next.saved_registers;
}
