#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use kfs::boot::MultibootInfo;
use kfs::{
    kassert, kassert_eq,
    vmm::{
        self,
        allocators::{
            backend::buddy::BUDDY_ALLOCATOR_SIZE,
            kmalloc::{buddy_allocator_alloc, buddy_allocator_free},
        },
        paging::PAGE_SIZE,
    },
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info)
}

#[test_case]
fn fragmentation_stress_test() -> Result<(), &'static str> {
    const MAX_ALLOCS: usize = 256;
    let mut ptrs = [(core::ptr::null(), 0); MAX_ALLOCS];

    for pp in ptrs.iter_mut() {
        if let Ok(p) = buddy_allocator_alloc(8192) {
            *pp = (p, 64);
        } else {
            break;
        }
    }

    for i in (0..MAX_ALLOCS).step_by(2) {
        if !ptrs[i].0.is_null() {
            let _ = buddy_allocator_free(ptrs[i].0);
            ptrs[i] = (core::ptr::null(), 0);
        }
    }

    for i in (0..MAX_ALLOCS).step_by(2) {
        if let Ok(p) = buddy_allocator_alloc(16384) {
            ptrs[i] = (p, 128);
        }
    }

    verify_no_overlaps(&ptrs)?;

    for p in ptrs {
        if !p.0.is_null() {
            let _ = buddy_allocator_free(p.0);
        }
    }

    Ok(())
}

#[test_case]
fn mixed_size_stress_test() -> Result<(), &'static str> {
    const MAX_ALLOCS: usize = 128;
    let mut ptrs = [(core::ptr::null(), 0); MAX_ALLOCS];
    let sizes = [4096, 8192, 16384, 32768, 32768 * 2, 32768 * 4];

    for i in 0..MAX_ALLOCS {
        let size = sizes[i % sizes.len()];
        if let Ok(p) = buddy_allocator_alloc(size) {
            ptrs[i] = (p, size);
        } else {
            break;
        }
    }

    verify_no_overlaps(&ptrs)?;

    for i in (0..MAX_ALLOCS).rev() {
        if !ptrs[i].0.is_null() {
            let _ = buddy_allocator_free(ptrs[i].0);
        }
    }

    Ok(())
}

#[test_case]
fn alternating_alloc_free_stress() -> Result<(), &'static str> {
    const ITERATIONS: usize = 100;
    const POOL_SIZE: usize = 32;
    let mut ptrs = [(core::ptr::null::<u8>(), 0); POOL_SIZE];
    let mut pattern = 0;

    for _ in 0..ITERATIONS {
        for i in 0..8 {
            let idx = (pattern + i) % POOL_SIZE;
            if ptrs[idx].0.is_null()
                && let Ok(p) = buddy_allocator_alloc(4096)
            {
                ptrs[idx] = (p, 4096);
            }
        }

        for i in 0..4 {
            let idx = (pattern + i) % POOL_SIZE;
            if !ptrs[idx].0.is_null() {
                let _ = buddy_allocator_free(ptrs[idx].0);
                ptrs[idx] = (core::ptr::null(), 0);
            }
        }

        pattern = (pattern + 3) % POOL_SIZE;

        verify_no_overlaps(&ptrs)?;
    }

    for p in ptrs {
        if !p.0.is_null() {
            let _ = buddy_allocator_free(p.0);
        }
    }

    Ok(())
}

#[test_case]
fn write_verify_test() -> Result<(), &'static str> {
    const NUM_BLOCKS: usize = 32;
    let mut ptrs = [(core::ptr::null(), 0); NUM_BLOCKS];

    for (i, pp) in ptrs.iter_mut().enumerate() {
        if let Ok(p) = buddy_allocator_alloc(4096) {
            *pp = (p, 4096);

            unsafe {
                let slice = core::slice::from_raw_parts_mut(p, 4096);
                for byte in slice.iter_mut() {
                    *byte = i as u8;
                }
            }
        }
    }

    for (i, pp) in ptrs.iter_mut().enumerate() {
        if !pp.0.is_null() {
            unsafe {
                let slice = core::slice::from_raw_parts(pp.0, 4096);
                for &byte in slice.iter() {
                    assert!(byte == i as u8, "Memory corruption detected");
                }
            }
        }
    }

    verify_no_overlaps(&ptrs)?;

    for p in ptrs {
        if !p.0.is_null() {
            let _ = buddy_allocator_free(p.0);
        }
    }

    Ok(())
}

fn verify_no_overlaps(ptrs: &[(*const u8, usize)]) -> Result<(), &'static str> {
    for (i, &p) in ptrs.iter().enumerate() {
        if p.0.is_null() {
            continue;
        }

        let (p_start, p_end) = (p.0 as usize, p.0 as usize + p.1);

        for (j, &pp) in ptrs.iter().enumerate() {
            if i == j || pp.0.is_null() {
                continue;
            }

            let (pp_start, pp_end) = (pp.0 as usize, pp.0 as usize + pp.1);

            assert!(
                !(p_start < pp_end && pp_start < p_end),
                "Overlapping allocations detected at indices {} and {}",
                i,
                j
            );
        }
    }
    Ok(())
}

#[test_case]
fn no_overlapping_allocations() -> Result<(), &'static str> {
    let mut ptrs = [(core::ptr::null(), 0); 20];
    let mut size = 4096;
    let mut i = 0;

    while let Ok(p) = buddy_allocator_alloc(size) {
        ptrs[i] = (p, size);
        i += 1;
        size *= 2;
    }

    for p in ptrs {
        for pp in ptrs {
            let (pp_start, pp_end) = (pp.0 as usize, pp.0 as usize + pp.1);
            let (p_start, p_end) = (p.0 as usize, p.0 as usize + p.1);
            if pp == p {
                continue;
            }
            assert!(!(p_start < pp_end && pp_start < p_end), "Buddy allocator returned overlapping memory addresses");
        }
    }

    for p in ptrs {
        if !p.0.is_null() {
            let _ = buddy_allocator_free(p.0);
        }
    }

    Ok(())
}

#[test_case]
fn full_cache_usable() -> Result<(), &'static str> {
    let mut ptrs = [core::ptr::null(); 8];
    for p in ptrs.iter_mut() {
        let ptr = buddy_allocator_alloc(BUDDY_ALLOCATOR_SIZE / 8).map_err(|_| "Allocation failed when it should have been able to service the request")?;

        *p = ptr;
    }

    for p in ptrs {
        let _ = buddy_allocator_free(p);
    }

    Ok(())
}

#[test_case]
fn alloc_free_alloc() -> Result<(), &'static str> {
    let p1 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Allocation failed")?;
    buddy_allocator_free(p1).map_err(|_| "Free failed")?;

    let p2 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Allocation failed")?;
    let p3 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Allocation failed")?;

    kassert_eq!(p1, p2);
    kassert!(p1 != p3, "buddy_allocator_alloc allocated the same address twice");

    buddy_allocator_free(p1).map_err(|_| "Free failed")?;
    buddy_allocator_free(p3).map_err(|_| "Free failed")?;

    let p = buddy_allocator_alloc(BUDDY_ALLOCATOR_SIZE).map_err(|_| "Allocation failed")?;
    buddy_allocator_free(p).map_err(|_| "Free failed")?;

    Ok(())
}

#[test_case]
fn alloc_full_size() -> Result<(), &'static str> {
    let ptr = buddy_allocator_alloc(BUDDY_ALLOCATOR_SIZE).map_err(|_| "Could not allocate full size of Buddy Allocator buffer")?;

    buddy_allocator_free(ptr).map_err(|_| "Free failed")?;

    Ok(())
}

#[test_case]
fn alloc_page_size() -> Result<(), &'static str> {
    let p1 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Could not allocate full size of Buddy Allocator buffer")?;
    let p2 = buddy_allocator_alloc(PAGE_SIZE).map_err(|_| "Could not allocate full size of Buddy Allocator buffer")?;

    kassert_eq!(p2 as usize - p1 as usize, PAGE_SIZE);

    buddy_allocator_free(p1).map_err(|_| "Free failed")?;
    buddy_allocator_free(p2).map_err(|_| "Free failed")?;

    Ok(())
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kmain(_magic: usize, info: &MultibootInfo) {
    use kfs::{
        arch, qemu,
        vmm::{allocators::kmalloc::KERNEL_ALLOCATOR, paging::init::init_memory},
    };

    arch::x86::gdt::init();
    arch::x86::idt::init();

    init_memory(info);

    #[allow(static_mut_refs)]
    if vmm::allocators::kmalloc::init_buddy_allocator(unsafe { &mut KERNEL_ALLOCATOR }).is_err() {
        panic!("Failed to initialize buddy allocator");
    }

    test_main();

    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
