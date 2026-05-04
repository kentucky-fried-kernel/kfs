use crate::{
    arch::x86::vmm::{
        init::load_page_directory,
        page::{self, PAGE_DIRECTORY_SIZE, PAGE_TABLE_SIZE, PageDirectory, PageDirectoryEntry, PageTable, PageTableEntry},
        state::{PAGE_DIRECTORY_KERNEL, PAGE_DIRECTORY_KERNEL_BOOT, PAGE_TABLES_KERNEL, PAGE_TABLES_KERNEL_SIZE},
    },
    boot::{KERNEL_BASE, STACK, STACK_SIZE},
    serial, serial_println,
};

const PAGE_TABLE_SIZE_USER: usize = (PAGE_TABLE_SIZE / 4) * 3;

#[derive(Clone, Debug)]
pub struct Addressspace {
    pub page_directory: PageDirectory,
    pub page_tables: [[PageTableEntry; PAGE_TABLE_SIZE]; PAGE_TABLE_SIZE_USER],
}

const TABLE_KERNEL_PAGE_TABLE_OFFSET: usize = 768;

impl Addressspace {
    pub fn empty() -> Self {
        let mut page_directory: PageDirectory = PageDirectory([PageDirectoryEntry::empty(); PAGE_DIRECTORY_SIZE]);

        // serial_println!("{:?}", PAGE_DIRECTORY_KERNEL);
        let page_directory_kernel = PAGE_DIRECTORY_KERNEL.lock().expect(
            "failed to lock
        PAGE_DIRECTORY_KERNEL",
        );

        for i in 0..PAGE_TABLE_SIZE {
            page_directory[i] = page_directory_kernel[i];
            if i >= (PAGE_TABLE_SIZE / 4) * 3 && i < (PAGE_TABLE_SIZE / 4) * 3 + 20 {
                // serial_println!("index {}", i);
                // serial_println!("{:x}", u32::from(page_directory[i]) >> 12);
                // serial_println!("{:x}", u32::from(page_directory_kernel[i]) >> 12);
            }
        }

        Self {
            page_directory,
            page_tables: [[PageTableEntry::empty(); PAGE_TABLE_SIZE]; PAGE_TABLE_SIZE_USER],
        }
    }

    pub fn switch_page_directory(&self) {
        serial_println!("ahhhh {:x}", self.get_base_physical() as usize);

        let value: usize;
        unsafe {
            core::arch::asm!(
                "mov {}, cr3",
                out(reg) value,
                options(nostack, nomem)
            );
        }
        serial_println!("loaded {:x}", value);

        serial_println!("hello from heeee ------------------");
        unsafe {
            load_page_directory(self.get_base_physical());
        }

        serial_println!("hello from afterwards");
        let value: usize;
        unsafe {
            core::arch::asm!(
                "mov {}, cr3",
                out(reg) value,
                options(nostack, nomem)
            );
        }
        serial_println!("loaded {:x}", value);
    }

    pub fn get_base_physical(&self) -> *mut PageDirectory {
        let pt = PAGE_TABLES_KERNEL.lock().unwrap();
        let addr = ((&self.page_directory) as *const _ as usize);
        let pd_index = addr >> 22;
        let pt_index = (addr << 10) >> 22;

        let pte = pt[pd_index - 768][pt_index];
        let paddr = (u32::from(pte) >> 12) << 12;
        serial_println!("pte {:x}", paddr);
        paddr as *mut PageDirectory
    }
}
