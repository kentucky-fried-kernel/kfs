use core::{arch::asm, ffi::c_uint, hint::select_unpredictable, ptr};

use crate::{
    arch::x86::{idt::InterruptRegisters, interrupts::irq},
    keyboard::{Keyboard, layout::map_qwerty},
    serial::print_internal,
    serial_println,
    shell::Shell,
    terminal::{self, SCREEN},
    vmm::paging::{
        PAGE_SIZE,
        mmap::{Mode, mmap},
    },
};

pub fn init() {
    irq::install_handler(0, timer);
    irq::clear_mask(0);
}

fn sys_exit() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

static mut QUEUE: [Option<PCB>; 20] = [None; 20];
static mut PID_NEXT: u16 = 1;
static mut PID_RUNNING: Option<u16> = None;

fn find_next_pid() -> u16 {
    unsafe {
        #[allow(static_mut_refs)]
        for (pid, e) in QUEUE.iter().enumerate().skip(PID_RUNNING.unwrap() as usize + 1) {
            if let Some(_) = e {
                return pid as u16;
            }
        }
        #[allow(static_mut_refs)]
        for (pid, e) in QUEUE.iter().enumerate() {
            if let Some(_) = e {
                return pid as u16;
            }
        }
    }

    1
}

#[unsafe(no_mangle)]
extern "C" fn timer(mut _regs: *mut InterruptRegisters) -> u32 {
    unsafe {
        serial_println!("timer {:x}", (*_regs).esp);
    }

    unsafe {
        if let None = PID_RUNNING {
            PID_RUNNING = Some(1);
        } else {
            let pcb = &mut QUEUE[PID_RUNNING.unwrap() as usize].as_mut().unwrap();
            pcb.esp = _regs as *mut InterruptRegisters as u32;
        }
    }

    let pid_running_next = find_next_pid();
    serial_println!("pid running next {}", pid_running_next);

    unsafe {
        PID_RUNNING = Some(pid_running_next as u16);
        let pcb = &mut QUEUE[pid_running_next as usize].unwrap();

        let queued_signal = 'found: {
            for x in pcb.queue_signals {
                if let Some(v) = x {
                    break 'found Some(v);
                }
            }
            None
        };

        match queued_signal {
            Some(s) => {
                // if let Some(f) = pcb.signal_handlers[s] {
                // } else {
                //     return pcb.esp;
                // }
                0
            }
            None => {
                return pcb.esp;
            }
        }
    }
}

#[repr(u8)]
pub enum SignalDefaultBehaviour {
    Terminate,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Signal {
    Kill = 0,
}
type PCB = ProcessControlBlock;
#[derive(Clone, Copy)]
pub struct ProcessControlBlock {
    id: u16,
    esp: u32,
    stack_start: u32,
    stack_size: u32,
    queue_signals: [Option<Signal>; 20],
    signal_handlers: [Option<fn(signal: Signal)>; 32],
}

impl ProcessControlBlock {
    pub fn new(id: u16, esp: u32, stack_start: u32, stack_size: u32) -> Self {
        Self {
            id,
            esp,
            stack_start,
            stack_size,
            queue_signals: [const { None }; 20],
            signal_handlers: [const { None }; 32],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Memory {
    stack_pointer: u32,
}

pub fn forked_function() -> ! {
    #[allow(static_mut_refs)]
    let mut shell = Shell::default(
        unsafe { &mut terminal::SCREEN },
        Keyboard::new(crate::keyboard::layout::Layout::new(map_qwerty)),
    );
    shell.launch();

    sys_exit();
}

pub fn print_red() -> ! {
    loop {
        serial_println!("red");
        unsafe {
            crate::printkln!("red");
        }
        unsafe {
            asm!("hlt");
        }
    }
    sys_exit();
}
// 0xc01050a4
pub fn print_green() -> ! {
    loop {
        serial_println!("green");
        unsafe {
            crate::printkln!("green");
        }
        unsafe {
            asm!("hlt");
        }
    }
    sys_exit();
}

#[unsafe(no_mangle)]
pub fn to_be_forked() -> ! {
    let pid = sys_fork();

    if pid == 0 { print_green() } else { print_red() }
}

pub fn sys_execve(f: fn() -> !, stack_size: usize) -> Result<(), ()> {
    // recreate a stack
    let stack = mmap(
        None,
        stack_size,
        crate::vmm::paging::Permissions::ReadWrite,
        crate::vmm::paging::Access::User,
        &Mode::Scattered,
    );

    let stack = match stack {
        Ok(a) => a,
        Err(e) => {
            serial_println!("ERROR ERROR");
            return Err(());
        }
    };

    let stack_start = stack;
    serial_println!("memor {:x}", stack);
    serial_println!("memor {:x}", stack + stack_size);

    let stack = stack + stack_size;

    let stack = stack - size_of::<InterruptRegisters>();

    let regs: &mut InterruptRegisters = unsafe {
        let ptr = stack as *mut InterruptRegisters;
        &mut *ptr
    };

    regs.esp = stack as u32;
    regs.eip = f as u32 - 1;
    regs.cr2 = 0x10;
    // regs.csm = 0x10;
    regs.csm = 0x8;
    regs.eflags = 0x200;
    serial_println!("after");

    unsafe {
        let pcb = PCB::new(PID_NEXT, stack as u32, stack_start as u32, stack_size as u32);
        QUEUE[PID_NEXT as usize] = Some(pcb);
        PID_NEXT += 1;
    }

    Ok(())
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn sys_fork() -> u32 {
    core::arch::naked_asm!("push esp", "call sys_fork_internal", "add esp, 4", "ret")
}

type SignalHandler = fn(signal: Signal);

pub fn signal(signal: Signal, f: &SignalHandler) -> Option<SignalHandler> {
    let pid = unsafe { PID_RUNNING.unwrap() };

    let pcb = unsafe { &mut QUEUE[pid as usize].unwrap() };

    let signal_handler_prev = pcb.signal_handlers[signal as usize];
    pcb.signal_handlers[signal as usize] = Some(*f);
    signal_handler_prev
}

pub fn kill(pid: usize, signal: Signal) -> Result<(), ()> {
    let mut pcb_current = unsafe { &mut QUEUE[pid].unwrap() };

    for s in pcb_current.queue_signals.iter_mut() {
        if let None = s {
            *s = Some(signal);
            return Ok(());
        }
    }

    Err(())
}

#[unsafe(no_mangle)]
pub fn sys_fork_internal(return_eip: u32) -> usize {
    let esp_before_fork = return_eip + 4;
    let eip_to_return_fo = unsafe { *(return_eip as *mut u32) };

    serial_println!(" esp {:x}", return_eip);
    serial_println!("return ip {:x}", eip_to_return_fo);

    let pcb_current = unsafe { QUEUE[PID_RUNNING.unwrap() as usize].unwrap() };

    let stack_new_start = mmap(
        None,
        pcb_current.stack_size as usize,
        crate::vmm::paging::Permissions::ReadWrite,
        crate::vmm::paging::Access::User,
        &Mode::Scattered,
    )
    .unwrap();

    unsafe {
        ptr::copy_nonoverlapping(pcb_current.stack_start as *mut u8, stack_new_start as *mut u8, pcb_current.stack_size as usize);
    }

    let pid_next = unsafe { PID_NEXT };
    unsafe {
        PID_NEXT += 1;
    }

    let diff = esp_before_fork - pcb_current.stack_start;
    let new_esp = stack_new_start + diff as usize;

    let pcb_new = PCB::new(
        pid_next,
        new_esp as u32 - size_of::<InterruptRegisters>() as u32,
        stack_new_start as u32,
        pcb_current.stack_size,
    );

    let ir = new_esp - size_of::<InterruptRegisters>();

    let regs: &mut InterruptRegisters = unsafe {
        let ptr = ir as *mut InterruptRegisters;
        &mut *ptr
    };

    regs.eax = 0;
    regs.esp = new_esp as u32;
    regs.eip = eip_to_return_fo as u32;
    regs.cr2 = 0x10;
    // regs.csm = 0x10;
    regs.csm = 0x8;
    regs.eflags = 0x200;

    unsafe {
        QUEUE[pid_next as usize] = Some(pcb_new);
    }
    // getpid
    // get information about the own process
    // duplicate memory
    //
    serial_println!("forked!!!!!");
    return pid_next as usize;
}
