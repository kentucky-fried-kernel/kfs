use core::{arch::asm, ffi::c_uint, hint::select_unpredictable};

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

#[unsafe(no_mangle)]
extern "C" fn timer(mut _regs: *mut InterruptRegisters) -> u32 {
    unsafe {
        serial_println!("timer {:x}", (*_regs).esp);
    }

    if unsafe { PID_RUNNING == None } {
        unsafe {
            PID_RUNNING = Some(2);
        }
    } else {
        unsafe {
            let pcb = &mut QUEUE[PID_RUNNING.unwrap() as usize].as_mut().unwrap();
            pcb.esp = _regs as *mut InterruptRegisters as u32;
        }
    }

    let pid_running_next = unsafe { if PID_RUNNING.unwrap() == 1 { 2 } else { 1 } };
    serial_println!("{}", pid_running_next);

    unsafe {
        PID_RUNNING = Some(pid_running_next as u16);
        let pcb = QUEUE[pid_running_next].unwrap();
        // let addr = &_regs as *const &mut InterruptRegisters;
        // let addr = addr as *mut u32;
        // *addr = pcb.esp;
        return pcb.esp;
    }
}

type PCB = ProcessControlBlock;
#[derive(Clone, Copy)]
pub struct ProcessControlBlock {
    id: u16,
    esp: u32,
}

impl ProcessControlBlock {
    pub fn new(id: u16, esp: u32) -> Self {
        Self { id, esp }
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
    serial_println!("after");

    unsafe {
        let pcb = PCB::new(PID_NEXT, stack as u32);
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
