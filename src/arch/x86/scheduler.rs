#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
use core::arch::naked_asm;

use alloc::vec::Vec;

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        interrupts::irq,
        kernel_mutex::KernelMutex,
        syscall::sys_exit,
        vmm::process::{Process, ProcessState, Scheduler},
    },
    binaries::{memory_seperation::memory_seperation_test, signals::signal_print_test, wait::wait_test},
    serial_println,
    signals::Action,
};
// #[unsafe(naked)]
// #[allow(unused)]
// extern "C" fn program_exit() {
//     naked_asm!("aaa:", "mov eax, 60", "int 0x80", "jmp aaa");
// }

//
// #[unsafe(naked)]
// extern "C" fn program_ipc_test() {
//     naked_asm!(
//         "mov eax, 99",
//         "int 0x80",
//         "mov ebx, eax",
//         "mov eax, 42",
//         "int 0x80",
//         // --- socket_create() -> eax = fd ---
//         "mov eax, 5",
//         "int 0x80",
//         // save fd in esi (callee-saved-ish for our purposes; nothing clobbers
//         // it until we use it again).
//         "mov esi, eax",
//         //
//         // --- fork() ---
//         "mov eax, 57",
//         "int 0x80",
//         //
//         //
//         // branch on eax: 0 = child, nonzero = parent
//         "test eax, eax",
//         "jz child",
//         "mov edi, eax",
//         // edi is now the child_id
//
//         // Store pid at [0x2000] so we have a stable address to pass as buf.
//         "mov dword ptr [0x2000], 0x696969",
//         // socket_write(fd=esi, buf=0x2000, len=4)
//         "mov eax, 8",
//         "mov ebx, esi",
//         "mov ecx, 0x2000",
//         "mov edx, 4",
//         "int 0x80",
//         "mov ebx, eax",
//         "mov eax, 42",
//         "int 0x80",
//         // exit(0). ebx = 0 so the parent's exit log is unambiguous.
//         "looping:",
//         "jmp looping",
//         // ===== CHILD =====
//         "child:",
//         // Busy-loop on socket_read until it returns > 0.
//         // (No blocking read yet, so we spin.)
//         "retry:",
//         "mov eax, 7",
//         "mov ebx, esi",
//         "mov ecx, 0x2000",
//         "mov edx, 4",
//         "int 0x80",
//         // eax = bytes read. If 0, retry.
//         "test eax, eax",
//         "jz retry",
//         // Load the received value into ebx and exit, so sys_exit prints it.
//         "mov ebx, [0x2000]",
//         //
//         "mov eax, 42",
//         "int 0x80",
//         //
//         "mov eax, 57",
//         "int 0x80",
//         "mov eax, 60",
//         "int 0x80",
//     );
// }
//
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
    let p = Process::new(&wait_test(), super::vmm::process::Parent::Root, false);
    SCHEDULER.lock().unwrap().spawn(p);

    irq::install_handler(0, timer);
    irq::clear_mask(0);
}

pub static SCHEDULER: KernelMutex<Scheduler> = KernelMutex::new(Scheduler::new());

#[allow(clippy::missing_panics_doc)]
pub extern "C" fn timer(regs: &mut InterruptRegisters) {
    serial_println!("timer");
    let mut scheduler = SCHEDULER.lock().expect("timer | failed to lock SCHEDULER");
    // first save the registers if there was a running process
    if let Some(p) = scheduler.current() {
        p.saved_registers = *regs;
    }

    // find the next process
    loop {
        let next = scheduler.schedule().expect("could not lock SCHEDULER");
        if next.state == ProcessState::Waiting {
            if let Some(child_stopped_id) = next.children_stopped.pop() {
                next.state = crate::arch::x86::vmm::process::ProcessState::Running;
                next.addressspace.load();
                *regs = next.saved_registers;
                regs.eax = child_stopped_id as u32;
                return;
            }
            continue;
        }
        if next.state == ProcessState::Blocked {
            continue;
        }

        if let Some(signal) = next.signals_queued.pop() {
            match next.signal_handlers.get(signal) {
                Action::Terminate => {
                    let _ = next;
                    drop(scheduler);
                    sys_exit(regs);
                    return;
                }
                Action::Ignore => {}
                Action::Continue => {
                    next.state = crate::arch::x86::vmm::process::ProcessState::Running;
                    continue;
                }
                Action::Stop => {
                    next.state = crate::arch::x86::vmm::process::ProcessState::Blocked;
                    continue;
                }
                Action::Handler(vaddr) => {
                    if next.signal_handler_saved_registers.is_some() {
                        next.signals_queued.push(signal);
                        next.addressspace.load();
                        *regs = next.saved_registers;
                        return;
                    }
                    next.enter_signal_handler(vaddr);
                }
            }
        }

        next.addressspace.load();
        *regs = next.saved_registers;
        break;
    }
}
