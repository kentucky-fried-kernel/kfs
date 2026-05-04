use core::ptr::write_volatile;

use alloc::vec::Vec;

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        interrupts::irq,
        kernel_mutex::KernelMutex,
        vmm::{
            PAGE_SIZE,
            addressspace::Addressspace,
            page::{PageDirectoryEntry, PageTableEntry},
            state::PAGE_ALLOCATOR,
        },
    },
    boot::KERNEL_BASE,
    serial_println,
};

#[derive(Clone, Debug)]
pub struct Segment {
    pub offset: Option<usize>,
    pub vaddr: usize,
    pub size: usize,
}

#[derive(Clone, Debug)]
pub struct Binary {
    pub segments: Vec<Segment>,
    pub entry: usize,
    pub stack: usize,
}

#[derive(Clone, Debug)]
pub enum Permissions {
    Read,
    ReadWrite,
}

type VMA = VirtualMemoryArea;

#[derive(Debug)]
pub struct VirtualMemoryArea {
    pub start: usize,
    pub size: usize,
    pub permissions: Permissions,
}

impl VMA {
    pub fn new(start: usize, size: usize, permissions: Permissions) -> Self {
        Self { start, size, permissions }
    }
}

#[derive(Debug)]
pub enum Status {
    Running,
    Stopped,
}
#[derive(Debug)]
pub struct Process {
    pub binary: Binary,
    pub regs: InterruptRegisters,
    pub vmas: Vec<VMA>,
    pub space: Addressspace,
    pub status: Status,
}

impl Process {
    pub fn new(binary: Binary) -> Self {
        Self {
            binary: binary.clone(),
            regs: InterruptRegisters {
                cr2: 0,
                ds: 0x23,
                edi: 0,
                esi: 0,
                ebp: 0,
                esp: 0,
                ebx: 0,
                edx: 0,
                ecx: 0,
                eax: 0,
                intno: 0,
                err_code: 0,
                eip: binary.entry as u32,
                csm: 0x1B,
                eflags: 0x200,
                useresp: binary.stack as u32,
                ss: 0x23,
            },
            vmas: binary
                .segments
                .iter()
                .map(|s| VMA {
                    start: s.vaddr,
                    size: s.size,
                    permissions: Permissions::ReadWrite,
                })
                .collect(),
            space: Addressspace::empty(),
            status: Status::Running,
        }
    }

    pub fn page_fault(&mut self, regs: &InterruptRegisters) {
        let addr = regs.cr2 as usize;
        let mut allowed = false;
        for vma in self.vmas.iter() {
            if vma.start <= addr && vma.start + vma.size > addr {
                allowed = true;
            }
        }
        if !allowed {
            return;
        }

        self.map_page(addr);

        let value: usize;
        unsafe {
            core::arch::asm!(
                "mov {}, cr3",
                out(reg) value,
                options(nostack, nomem)
            );
        }
        serial_println!("loaded {:x}", value);

        let a = 0x2000_0000 as *mut u8;
        serial_println!("hello");
        let b = 3;
        unsafe {
            write_volatile(a, b);
        }
        serial_println!("hello");

        for segment in self.binary.segments.iter() {
            if let Some(offset) = segment.offset {
                if segment.vaddr <= addr && segment.vaddr + segment.size > addr {
                    let diff = addr - segment.vaddr;
                    unsafe {
                        serial_println!("offset {:x}", offset);
                        serial_println!("vaddr {:x}", addr);
                        serial_println!("diff {:x}", diff);
                        core::ptr::copy_nonoverlapping((offset + diff) as *mut u8, addr as *mut u8, PAGE_SIZE);
                        serial_println!("arstarst");
                    }
                }
            }
        }
    }

    fn map_page(&mut self, vaddr: usize) -> Result<(), ()> {
        assert!(vaddr < KERNEL_BASE);
        let paddr = PAGE_ALLOCATOR.lock().expect("failed to lock PAGE_ALLOCATOR").alloc(PAGE_SIZE);
        let paddr = match paddr {
            Some(p) => p as usize,
            None => return Err(()),
        };

        let page_directory_index = vaddr >> 22;
        let page_table_paddr = &mut self.space.page_tables[page_directory_index] as *const _ as usize - KERNEL_BASE;
        let page_table_paddr_test = &mut self.space.page_directory[0] as *const _ as usize - KERNEL_BASE;

        let page_table_index = (vaddr >> 12) & 0b11_11_11_11_11;
        let page_directory_entry: &mut PageDirectoryEntry = &mut self.space.page_directory[page_directory_index];
        let page_table_entry: &mut PageTableEntry = &mut self.space.page_tables[page_directory_index][page_table_index];

        let mut pde = PageDirectoryEntry::empty();
        pde.set_address((page_table_paddr >> 12) as u32);
        pde.set_read_write(1);
        pde.set_user_supervisor(1);
        pde.set_present(1);
        *page_directory_entry = pde;

        let mut pte = PageTableEntry::empty();
        pte.set_address((paddr >> 12) as u32);
        pte.set_read_write(1);
        pte.set_user_supervisor(1);
        pte.set_present(1);
        *page_table_entry = pte;

        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack));
        }

        Ok(())
    }
}

pub static QUEUE: KernelMutex<Vec<Process>> = KernelMutex::new(Vec::new());

pub fn init() {
    let mut init = Binary {
        segments: Vec::new(),
        entry: 0x2000_0000,
        stack: 0x2000_4000,
    };
    init.segments.push(Segment {
        offset: Some(init_function as usize),
        vaddr: 0x2000_0000,
        size: 0x1000,
    });

    init.segments.push(Segment {
        offset: None,
        vaddr: 0x2000_1000,
        size: 0x3000,
    });

    let init = Process::new(init);

    let mut queue = QUEUE.lock().expect("tried to lock queue");
    queue.push(init);

    serial_println!("--------------------------------");
    let proc = &mut queue[0];
    serial_println!("now thats the real onoe {:x}", proc.space.get_base_physical() as usize);
    proc.space.switch_page_directory();
    drop(queue);

    irq::install_handler(0, timer);
    irq::clear_mask(0);
}

extern "C" fn timer(regs: &mut InterruptRegisters) {
    serial_println!("ss {:x}", regs.ss);
    serial_println!("eip {:x}", regs.eip);
    serial_println!();
    let queue = QUEUE.lock().expect("couldn't lock queue");
    let process = queue.iter().next().unwrap();
    *regs = process.regs;
    drop(queue);
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn init_function() {
    core::arch::naked_asm!("aaaa:", "mov eax, 0x688302", "int 0x80", "jmp aaaa")
}
