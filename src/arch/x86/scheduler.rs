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
    binaries::{
        fork_bomb::fork_bomb_test, memory_protection::memory_protection_test, signals::signal_print_test, sockets::sockets_test, stack::stack_test,
        super_user::super_user_test, wait::wait_test,
    },
    serial_println,
    signals::Action,
};

#[allow(clippy::missing_panics_doc)]
pub fn init() {
    let b = stack_test();
    let p = Process::new(&b, super::vmm::process::Parent::Root, true);
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
