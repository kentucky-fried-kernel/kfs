use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        scheduler::{SCHEDULER, timer},
        vmm::process::Process,
    },
    serial_println,
};

pub extern "C" fn sys_exit(regs: &mut InterruptRegisters) {
    serial_println!("exited while ebx was {}", regs.ebx);
    let mut scheduler = SCHEDULER.lock().expect("timer | failed to lock SCHEDULER");

    let pid = scheduler.current().expect("sys_exit | no process running").pid;

    scheduler.exit(pid);

    let next = scheduler.schedule().expect("no process to run");

    next.state = crate::arch::x86::vmm::process::ProcessState::Running;
    next.addressspace.load();
    *regs = next.saved_registers;
}

pub fn sys_fork(regs: &mut InterruptRegisters) {
    serial_println!("forked");
    let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
    let parent = scheduler.current().expect("sys_fork | no process running");
    parent.saved_registers = *regs;
    let mut child = Process::from_process(parent);
    child.saved_registers.eax = 0;

    drop(parent);
    let child_pid = scheduler.spawn(child);

    let parent = scheduler.current().expect("sys_fork | no process running");
    parent.saved_registers.eax = child_pid as u32;
    regs.eax = child_pid as u32;
}

pub fn sys_getpid(regs: &mut InterruptRegisters) {
    let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
    let cur = scheduler.current().expect("sys_fork | no process running");
    regs.eax = cur.pid as u32;
}

pub fn syscall(regs: &mut InterruptRegisters) {
    match regs.eax {
        35 => timer(regs),
        57 => sys_fork(regs),
        60 => sys_exit(regs),
        _ => regs.eax = u32::MAX,
    }
}
