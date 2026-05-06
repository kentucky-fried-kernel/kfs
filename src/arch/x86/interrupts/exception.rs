use core::{intrinsics::copy_nonoverlapping, ptr::write_volatile};

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        scheduler::{Permissions, SCHEDULER, timer},
        vmm::{
            PAGE_SIZE,
            addressspace::Addressspace,
            process::{Process, VMA},
            state::PAGE_ALLOCATOR,
        },
    },
    serial_println,
};

macro_rules! no_err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push 0", "push {0}", "jmp exception_common_stub", const $nb);
        }
    };
}

macro_rules! err_stub {
    ($func: ident, $nb: expr) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $func() {
            core::arch::naked_asm!("cli", "push {0}", "jmp exception_common_stub", const $nb);
        }
    };
}

pub mod _stubs {
    no_err_stub!(exception_stub_0, 0);
    no_err_stub!(exception_stub_1, 1);
    no_err_stub!(exception_stub_2, 2);
    no_err_stub!(exception_stub_3, 3);
    no_err_stub!(exception_stub_4, 4);
    no_err_stub!(exception_stub_5, 5);
    no_err_stub!(exception_stub_6, 6);
    no_err_stub!(exception_stub_7, 7);
    err_stub!(exception_stub_8, 8);
    no_err_stub!(exception_stub_9, 9);
    err_stub!(exception_stub_10, 10);
    err_stub!(exception_stub_11, 11);
    err_stub!(exception_stub_12, 12);
    err_stub!(exception_stub_13, 13);
    err_stub!(exception_stub_14, 14);
    no_err_stub!(exception_stub_15, 15);
    no_err_stub!(exception_stub_16, 16);
    err_stub!(exception_stub_17, 17);
    no_err_stub!(exception_stub_18, 18);
    no_err_stub!(exception_stub_19, 19);
    no_err_stub!(exception_stub_20, 20);
    no_err_stub!(exception_stub_21, 21);
    no_err_stub!(exception_stub_22, 22);
    no_err_stub!(exception_stub_23, 23);
    no_err_stub!(exception_stub_24, 24);
    no_err_stub!(exception_stub_25, 25);
    no_err_stub!(exception_stub_26, 26);
    no_err_stub!(exception_stub_27, 27);
    no_err_stub!(exception_stub_28, 28);
    no_err_stub!(exception_stub_29, 29);
    err_stub!(exception_stub_30, 30);
    no_err_stub!(exception_stub_31, 31);
    no_err_stub!(syscall_stub, 128);
}

#[macro_export]
macro_rules! exception_stubs {
    () => {
        &[
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_0 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_1 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_2 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_3 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_4 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_5 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_6 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_7 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_8 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_9 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_10 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_11 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_12 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_13 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_14 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_15 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_16 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_17 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_18 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_19 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_20 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_21 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_22 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_23 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_24 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_25 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_26 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_27 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_28 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_29 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_30 as *const () as usize,
            $crate::arch::x86::interrupts::exception::_stubs::exception_stub_31 as *const () as usize,
        ]
    };
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn exception_common_stub(intno: u32, stack_ptr: u32) {
    core::arch::naked_asm!(
        "pusha",
        "mov eax, ds",
        "push eax",
        "mov eax, cr2",
        "push eax",
        //
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        //
        "push esp",
        "call exception_handler",
        //
        "add esp, 8",
        "pop ebx",
        "mov ds, bx",
        "mov es, bx",
        "mov fs, bx",
        "mov gs, bx",
        //
        "popa",
        "add esp, 8",
        "iretd"
    )
}

const EXCEPTION_MESSAGE: &[&str] = &[
    "Division By Zero",
    "Debug",
    "Non-Maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
];

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
fn sys_fork(regs: &mut InterruptRegisters) {
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

fn syscall(regs: &mut InterruptRegisters) {
    match regs.eax {
        60 => sys_exit(regs),
        35 => timer(regs),
        57 => sys_fork(regs),
        _ => regs.eax = u32::MAX,
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn exception_handler(regs: &mut InterruptRegisters) {
    match regs.intno {
        13 => sys_exit(regs),
        14 => page_fault(regs),
        0..32 => {
            serial_println!("\nEXCEPTION {}: {}", regs.intno, EXCEPTION_MESSAGE[regs.intno as usize]);
            sys_exit(regs);
        }
        0x80 => syscall(regs),
        _ => panic!("{regs:?}"),
    }
}

fn page_fault(regs: &mut InterruptRegisters) {
    serial_println!("page_fault: at addr {:x}", regs.cr2);
    let mut scheduler = SCHEDULER.lock().expect("page_fault | could not lock SCHEDULER");

    let addr = regs.cr2 as usize;
    let addr = ((addr + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;
    let process = scheduler.current().expect("page_fault | no process running");

    for vma in &process.vmas {
        let access_allowed = vma.start <= addr && vma.start + vma.size > addr;
        if access_allowed {
            if let Ok(_) = alloc_page(addr, &mut process.addressspace, &vma) {
                return;
            }
        }
    }
    drop(scheduler);
    sys_exit(regs);
}

fn alloc_page(vaddr: usize, space: &mut Addressspace, vma: &VMA) -> Result<(), ()> {
    let mut alloc = PAGE_ALLOCATOR.lock().expect("page_fault | could not lock PAGE_ALLOCATOR");
    let paddr = match alloc.alloc(PAGE_SIZE) {
        Some(paddr) => paddr,
        None => {
            return Err(());
        }
    };

    let temp = 0xBFFF_F000 as *mut u8;

    if let Some(offset) = vma.offset {
        space.map(temp, paddr, Permissions::ReadWrite);
        let diff = vaddr - vma.start;
        unsafe {
            copy_nonoverlapping((offset + diff) as *mut u8, temp, PAGE_SIZE);
        }
        space.unmap(temp);
    }

    space.map(vaddr as *mut u8, paddr, vma.permissions);
    Ok(())
}
// pub struct VMA {
//     start: usize,
//     offset: Option<usize>,
//     size: usize,
//     permissions: Permissions,
// }
//
