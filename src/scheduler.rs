use core::{arch::asm, hint::select_unpredictable};

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

extern "C" fn timer(_regs: &mut InterruptRegisters) {
    serial_println!("timer {:x}", _regs.esp);

    if unsafe { PID_RUNNING == None } {
        unsafe {
            PID_RUNNING = Some(2);
        }
    } else {
        unsafe {
            let pcb = &mut QUEUE[PID_RUNNING.unwrap() as usize].as_mut().unwrap();
            pcb.regs = Some(*_regs);
            serial_println!("{:?}", pcb.regs);
        }
    }

    let pid_running_next = unsafe { if PID_RUNNING.unwrap() == 1 { 2 } else { 1 } };
    serial_println!("{}", pid_running_next);

    unsafe {
        PID_RUNNING = Some(pid_running_next as u16);
        let pcb = QUEUE[pid_running_next].unwrap();
        // asm!("sti");
        //
        match pcb.regs {
            Some(regs) => {
                serial_println!("hello");
                *_regs = regs;
            }
            None => {
                (*_regs).eip = pcb.start;
                (*_regs).esp = pcb.esp;
            }
        }
        serial_println!("esp {:x}", pcb.esp);
        serial_println!("esp {:x}", _regs.esp);
    }
    return;
}

type PCB = ProcessControlBlock;
#[derive(Clone, Copy)]
pub struct ProcessControlBlock {
    id: u16,
    start: u32,
    esp: u32,
    regs: Option<InterruptRegisters>,
}

impl ProcessControlBlock {
    pub fn new(id: u16, esp: u32, start: u32) -> Self {
        Self { id, start, esp, regs: None }
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

    serial_println!("memor {:x}", stack);
    serial_println!("memor {:x}", stack + stack_size);

    serial_println!("after");

    unsafe {
        let pcb = PCB::new(PID_NEXT, (stack + stack_size) as u32, f as u32);
        QUEUE[PID_NEXT as usize] = Some(pcb);
        PID_NEXT += 1;
    }

    Ok(())
}

pub fn sys_fork() {
    // getpid
    // get information about the own process
    // duplicate memory
    //
}
