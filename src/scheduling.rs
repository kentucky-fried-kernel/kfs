use alloc::vec::Vec;

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        kernel_mutex::KernelMutex,
        vmm::{addressspace::AddressSpace, init::load_page_directory},
    },
    serial_println,
};

pub static QUEUE: KernelMutex<Vec<Process>> = KernelMutex::new(Vec::new());

/// A kernel process with its own address space.
#[derive(Debug)]
pub struct Process {
    pub address_space: AddressSpace,
}

impl Process {
    fn new() -> Self {
        Process { address_space: AddressSpace::new() }
    }

    /// Called on a page fault originating from this process.
    pub fn page_fault(&mut self, regs: &mut InterruptRegisters) {
        serial_println!("page fault in process: eip={:x} cr2={:x}", regs.eip, regs.cr2);
    }
}

/// Initializes the scheduler: creates an initial process and switches to its
/// page directory.
///
/// Must be called after `vmm::init()` so that the heap allocator and the
/// kernel page directory are ready.
///
/// # Panics
/// Panics if the scheduler queue cannot be locked.
pub fn init() {
    let process = Process::new();

    // Capture the physical CR3 address before moving `process` into the queue.
    // The Box inside AddressSpace is a heap allocation whose address is stable
    // even after the Process struct is moved into the Vec.
    let cr3 = process.address_space.cr3();

    let mut queue = QUEUE.lock().expect("failed to lock QUEUE");
    queue.push(process);
    drop(queue);

    // Switch to the process's page directory.
    // cr3() already returns the physical address (vaddr - KERNEL_BASE), which
    // is what load_page_directory / CR3 expects.
    unsafe { load_page_directory(cr3) };
}
