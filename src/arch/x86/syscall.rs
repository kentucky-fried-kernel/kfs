#![allow(clippy::expect_used)]
#![allow(clippy::missing_panics_doc)]

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        scheduler::{SCHEDULER, timer},
        vmm::process::{Parent, Pid, Process},
    },
    serial_println,
    signals::{Action, Signal},
    socket::{SOCKETS, socket_close, socket_create, socket_read, socket_write},
};

pub extern "C" fn sys_exit(regs: &mut InterruptRegisters) {
    let mut scheduler = SCHEDULER.lock().expect("sys_exit | failed to lock SCHEDULER");

    let pid = scheduler.current().expect("sys_exit | no process running").pid;
    serial_println!("pid {} exited", pid);

    {
        let child = scheduler.table.get_mut(pid).expect("sys_exit | could not find exited process");
        let mut sockets = SOCKETS.lock().expect("sys_exit | could not lock SOCKETS");
        for socket_id in child.socket_fds.iter().flatten() {
            let _ = sockets.close(*socket_id);
        }
    }

    {
        let current = scheduler.table.get_mut(pid).expect("sys_exit | could not find exited process");
        let children = current.children.clone();
        let parent_id = match current.parent {
            Parent::Root => panic!("root process exited"),
            Parent::Pid(id) => id,
        };
        let _ = current;

        for c in &children {
            let child = scheduler.table.get_mut(*c).expect("sys_exit | could not find exited process");
            child.parent = Parent::Pid(parent_id);
            let parent = scheduler.table.get_mut(parent_id).expect("sys_exit | could not find exited process");
            parent.children.push(*c);
        }
        let parent = scheduler.table.get_mut(parent_id).expect("sys_exit | parent not in table");
        parent.children.retain(|&c| c != pid);
        parent.children_stopped.push(pid);
    }

    scheduler.exit(pid);

    let next = scheduler.schedule().expect("no process to run");

    next.addressspace.load();
    *regs = next.saved_registers;
}

pub fn sys_fork(regs: &mut InterruptRegisters) {
    serial_println!("forked");
    let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
    let parent = scheduler.current().expect("sys_fork | no process running");
    parent.saved_registers = *regs;
    let mut child = Process::from_process(parent);

    for socket_id in child.socket_fds.iter().flatten() {
        let mut sockets = SOCKETS.lock().expect("sys_fork | could not lock SOCKETS");
        let _ = sockets.add_ref(*socket_id);
    }

    child.saved_registers.eax = 0;

    let _ = parent;
    let child_pid = scheduler.spawn(child);

    let parent = scheduler.current().expect("sys_fork | no process running");
    parent.children.push(child_pid);
    parent.saved_registers.eax = child_pid as u32;
    regs.eax = child_pid as u32;
}

pub fn sys_getpid(regs: &mut InterruptRegisters) {
    let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
    let cur = scheduler.current().expect("sys_fork | no process running");
    regs.eax = cur.pid as u32;
}

pub fn sys_socket_create(regs: &mut InterruptRegisters) {
    let sid = socket_create();

    let mut scheduler = SCHEDULER.lock().expect("sys_socket_create | could not lock SCHEDULER");
    let process = scheduler.current().expect("sys_socket_create | no process running");
    let fd = process.install_socket(sid);
    regs.eax = fd as u32;
}

pub fn sys_socket_close(regs: &mut InterruptRegisters) {
    let fd = regs.ebx as usize;

    let sid = {
        let mut scheduler = SCHEDULER.lock().expect("sys_socket_close | could not lock SCHEDULER");
        let process = scheduler.current().expect("sys_socket_close | no process running");
        process.remove_socket(fd)
    };

    regs.eax = match sid {
        None => u32::MAX,
        Some(sid) => match socket_close(sid) {
            Ok(()) => 0,
            Err(_) => u32::MAX,
        },
    };
}

pub fn sys_socket_write(regs: &mut InterruptRegisters) {
    let fd = regs.ebx as usize;
    let buf = regs.ecx as usize;
    let len = regs.edx as usize;

    let (sid, vmas) = {
        let mut scheduler = SCHEDULER.lock().expect("sys_socket_write | could not lock SCHEDULER");
        let process = scheduler.current().expect("sys_socket_write | no process running");
        (process.resolve_socket(fd), process.vmas.clone())
    };

    let Some(sid) = sid else {
        regs.eax = u32::MAX;
        return;
    };

    regs.eax = match socket_write(sid, buf, len, &vmas) {
        Ok(n) => n as u32,
        Err(_) => u32::MAX,
    };
}

pub fn sys_socket_read(regs: &mut InterruptRegisters) {
    let fd = regs.ebx as usize;
    let buf = regs.ecx as usize;
    let len = regs.edx as usize;

    let (sid, vmas) = {
        let mut scheduler = SCHEDULER.lock().expect("sys_socket_read | could not lock SCHEDULER");
        let process = scheduler.current().expect("sys_socket_read | no process running");
        (process.resolve_socket(fd), process.vmas.clone())
    };

    let Some(sid) = sid else {
        regs.eax = u32::MAX;
        return;
    };

    regs.eax = match socket_read(sid, buf, len, &vmas) {
        Ok(n) => {
            serial_println!("read that many bytes {}", n);
            n as u32
        }
        Err(_) => u32::MAX,
    };
}

pub fn sys_am_super_user(regs: &mut InterruptRegisters) {
    let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
    let cur = scheduler.current().expect("sys_fork | no process running");
    regs.eax = match cur.super_user {
        true => 1,
        false => 2,
    }
}

pub fn sys_putnbr(regs: &mut InterruptRegisters) {
    let scheduler = SCHEDULER.lock().expect("could not lock SCHEDULER");
    serial_println!("Registers of pid: {}", scheduler.current.expect("no process running"));
    serial_println!("eax: {:#010x}", regs.eax);
    serial_println!("ebx: {:#010x}", regs.ebx);
    serial_println!("ecx: {:#010x}", regs.ecx);
    serial_println!("edx: {:#010x}", regs.edx);
    serial_println!("esi: {:#010x}", regs.esi);
    serial_println!("edi: {:#010x}", regs.edi);
    serial_println!("ebp: {:#010x}", regs.ebp);
    serial_println!("esp: {:#010x}", regs.esp);
    regs.eax = 0;
    serial_println!();
}

pub fn sys_wait(regs: &mut InterruptRegisters) {
    {
        let mut scheduler = SCHEDULER.lock().expect("sys_wait | could not lock SCHEDULER");
        let cur = scheduler.current().expect("sys_wait | no process running");
        cur.state = super::vmm::process::ProcessState::Waiting;
    }

    timer(regs);
}

pub fn sys_signal(regs: &mut InterruptRegisters) {
    let signal = match Signal::try_from(regs.ebx as u8) {
        Ok(signal) => signal,
        Err(_) => {
            regs.eax = u32::MAX;
            return;
        }
    };

    let vaddr = regs.ecx as usize;

    let mut scheduler = SCHEDULER.lock().expect("sys_signal | could not lock SCHEDULER");
    let cur = scheduler.current().expect("sys_signal | no process running");
    match cur.signal_handlers.set(signal, Action::Handler(vaddr)) {
        Ok(_) => regs.eax = 0,
        Err(_) => regs.eax = u32::MAX,
    }
}

pub fn sys_kill(regs: &mut InterruptRegisters) {
    let signal = match Signal::try_from(regs.ebx as u8) {
        Ok(signal) => signal,
        Err(_) => {
            regs.eax = u32::MAX;
            return;
        }
    };

    let mut scheduler = SCHEDULER.lock().expect("sys_signal | could not lock SCHEDULER");
    let process = {
        let pid = regs.ecx as Pid;
        match scheduler.table.get_mut(pid) {
            Some(process) => process,
            None => {
                regs.eax = u32::MAX - 1;
                return;
            }
        }
    };

    process.signals_queued.push(signal);
    regs.eax = 0;
}

pub fn sys_exit_signal_handler(regs: &mut InterruptRegisters) {
    let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
    let cur = scheduler.current().expect("sys_fork | no process running");
    cur.exit_signal_handler();
}

pub fn syscall(regs: &mut InterruptRegisters) {
    {
        let mut scheduler = SCHEDULER.lock().expect("sys_fork | could not lock SCHEDULER");
        let cur = scheduler.current().expect("sys_fork | no process running");
        serial_println!("syscall from pid {} with nbr {}", cur.pid, regs.eax);
        let _ = cur;
        drop(scheduler);
        match regs.eax {
            5 => sys_socket_create(regs),
            6 => sys_socket_close(regs),
            7 => sys_socket_read(regs),
            8 => sys_socket_write(regs),
            35 => timer(regs),
            42 => sys_putnbr(regs),
            57 => sys_fork(regs),
            60 => sys_exit(regs),
            70 => sys_signal(regs),
            71 => sys_kill(regs),
            72 => sys_exit_signal_handler(regs),
            98 => sys_wait(regs),
            99 => sys_am_super_user(regs),
            _ => regs.eax = u32::MAX,
        }
    }
    timer(regs);
}
