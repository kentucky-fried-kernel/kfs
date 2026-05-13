use core::ptr::copy_nonoverlapping;

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        scheduler::{Permissions, SCHEDULER},
        syscall::sys_exit,
        vmm::{PAGE_SIZE, addressspace::Addressspace, process::VMA, state::PAGE_ALLOCATOR},
    },
    serial_println,
};

pub fn page_fault(regs: &mut InterruptRegisters) {
    serial_println!("page_fault: at addr {:x}", regs.cr2);
    #[allow(clippy::missing_panics_doc)]
    let mut scheduler = SCHEDULER.lock().expect("page_fault | could not lock SCHEDULER");

    let addr = regs.cr2 as usize;
    let addr = ((addr + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;
    #[allow(clippy::missing_panics_doc)]
    let process = scheduler.current().expect("page_fault | no process running");

    for vma in &process.vmas {
        let access_allowed = vma.start <= addr && vma.start + vma.size > addr;
        if access_allowed && let Ok(()) = alloc_page(addr, &mut process.addressspace, vma) {
            return;
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

        // SAFETY:
        // temp is a valid address because it just got mapped.
        // offset + diff was validated when the binary was created.
        unsafe {
            copy_nonoverlapping((offset + diff) as *mut u8, temp, PAGE_SIZE);
        }
        space.unmap(temp);
    }

    space.map(vaddr as *mut u8, paddr, vma.permissions);
    Ok(())
}
