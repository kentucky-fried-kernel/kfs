use core::ptr::{copy_nonoverlapping, write_bytes};

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        scheduler::SCHEDULER,
        syscall::sys_exit,
        vmm::{PAGE_SIZE, addressspace::Addressspace, process::VMA, state::PAGE_ALLOCATOR},
    },
    binary::Permissions,
    serial_println,
};

pub fn page_fault(regs: &mut InterruptRegisters) {
    serial_println!("page_fault: at addr 0x{:x}", regs.cr2);
    #[allow(clippy::missing_panics_doc)]
    let mut scheduler = SCHEDULER.lock().expect("page_fault | could not lock SCHEDULER");

    #[allow(clippy::missing_panics_doc)]
    let process = scheduler.current().expect("page_fault | no process running");

    let addr = regs.cr2 as usize;
    let present = regs.err_code & 0b01 != 0;
    let write = regs.err_code & 0b10 != 0;

    for vma in &process.vmas {
        let access_allowed = vma.start <= addr && vma.start + vma.size > addr;
        if !access_allowed {
            continue;
        }

        if write && vma.permissions == Permissions::Read {
            break;
        }

        let addr = (addr / PAGE_SIZE) * PAGE_SIZE;
        if !present && alloc_page(addr, &mut process.addressspace, vma).is_ok() {
            return;
        }
        // && let Ok(()) = alloc_page(addr, &mut process.addressspace, vma)
    }
    serial_println!("process with id {} killed because of not allowed access at addr 0x{:x}", process.pid, addr);
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

    space.map(temp, paddr, Permissions::ReadWrite);

    let diff = vaddr - vma.start;
    match vma.offset {
        Some(offset) => {
            // SAFETY:
            // temp is a valid address because it just got mapped.
            // offset + diff was validated when the binary was created.
            unsafe {
                copy_nonoverlapping((offset + diff) as *const u8, temp, PAGE_SIZE);
            }
        }
        None => {
            // SAFETY:
            // temp is valid because we just mapped it.
            unsafe {
                write_bytes(temp, 0, PAGE_SIZE);
            }
        }
    }
    space.unmap(temp);

    space.map(vaddr as *mut u8, paddr, vma.permissions);
    Ok(())
}
