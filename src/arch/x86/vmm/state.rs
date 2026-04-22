use crate::arch::x86::vmm::{
    PAGE_SIZE,
    page::{PAGE_DIRECTORY_SIZE, PAGE_TABLE_SIZE, PageDirectory, PageDirectoryEntry, PageTable, PageTableEntry},
    page_allocator::{Node, ORDERS, PageAllocator},
};

use crate::arch::x86::kernel_mutex::KernelMutex;

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
/// Temporary Page Directory making it possible to execute in Kernel Space
/// (upper half/higher half) without the whole memory being setup
/// No page tables are needed because the whole 4mb are being mapped
/// through the page directory
/// This is also used to bootstrap the page_allocators until the first process
/// has its own {PageDirectory}
pub(super) static PAGE_DIRECTORY_KERNEL_BOOT: PageDirectory = {
    let mut dir: [PageDirectoryEntry; PAGE_DIRECTORY_SIZE] = [PageDirectoryEntry::from(0); PAGE_DIRECTORY_SIZE];

    dir[0] = PageDirectoryEntry::from((0 << 22) | 0b1000_0011);

    dir[768] = PageDirectoryEntry::from((0 << 22) | 0b1000_0011);
    dir[769] = PageDirectoryEntry::from((1 << 22) | 0b1000_0011);
    dir[770] = PageDirectoryEntry::from((2 << 22) | 0b1000_0011);
    dir[771] = PageDirectoryEntry::from((3 << 22) | 0b1000_0011);
    dir[772] = PageDirectoryEntry::from((4 << 22) | 0b1000_0011);
    dir[773] = PageDirectoryEntry::from((5 << 22) | 0b1000_0011);
    dir[774] = PageDirectoryEntry::from((6 << 22) | 0b1000_0011);
    dir[775] = PageDirectoryEntry::from((7 << 22) | 0b1000_0011);
    dir[776] = PageDirectoryEntry::from((8 << 22) | 0b1000_0011);

    PageDirectory(dir)
};

pub(super) static PAGE_DIRECTORY_KERNEL: KernelMutex<PageDirectory> = KernelMutex::new(PageDirectory([PageDirectoryEntry::empty(); PAGE_DIRECTORY_SIZE]));

pub(super) const PAGE_TABLES_KERNEL_SIZE: usize = PAGE_DIRECTORY_SIZE / 4; // Because the 4th GB in vm is used for kernel space only a 4th of the page tables are needed to represent kernel space
pub(super) static PAGE_TABLES_KERNEL: KernelMutex<[PageTable; PAGE_TABLES_KERNEL_SIZE]> =
    KernelMutex::new([PageTable([PageTableEntry::empty(); PAGE_TABLE_SIZE]); PAGE_TABLES_KERNEL_SIZE]);

// ---------------------------------------------------------------------------
// Page Allocator state
// ---------------------------------------------------------------------------
//
// The page allocator is a buddy allocator covering the full 4 GiB virtual
// address space. With 4 KiB pages that is 2^20 pages, so order 0 has 2^20
// entries, order 1 has 2^19, ..., order 20 has a single entry. The total
// entry count across every order is `2 * 2^20 - 1 = 2_097_151` entries, each
// 8 bytes (`Option<Node>` is `Option<NonZeroU64>`, niche-optimized), for a
// total of ~16 MiB of backing storage.
//
// Everything is initialized at compile time:
//   - every order's backing array is zero-filled `None`s (and `None` for `Option<NonZeroU64>` is
//     exactly the zero bit pattern, so these land in `.bss` automatically).
//   - order 20's single slot holds one `Node` representing the whole 4 GiB as a free block, and
//     `orders_head[20] = Some(0)` points at it.
//   - all other orders start empty, `orders_head[i < 20] = None`.
//
// `PageAllocator.orders` is a `[&'a mut [Option<Node>]; ORDERS]`. You can't
// take `&mut` to a `static mut` in const context the normal way, but
// `&mut *core::ptr::addr_of_mut!(STATIC)` is const-legal and produces a
// `&'static mut` slice reference with the right lifetime.
// Backing storage for each buddy-allocator order. Entry `i` at order `o`
// represents the block of `2^o` pages starting at page index `i * 2^o`.
// `None` means the block is either in-use or has been coalesced into a
// larger block; `Some(Node)` means it is free and part of that order's
// doubly-linked free list.
pub(super) static mut PAGE_ALLOCATOR_ORDER_00: [Option<Node>; 1 << 20] = [None; 1 << 20];
pub(super) static mut PAGE_ALLOCATOR_ORDER_01: [Option<Node>; 1 << 19] = [None; 1 << 19];
pub(super) static mut PAGE_ALLOCATOR_ORDER_02: [Option<Node>; 1 << 18] = [None; 1 << 18];
pub(super) static mut PAGE_ALLOCATOR_ORDER_03: [Option<Node>; 1 << 17] = [None; 1 << 17];
pub(super) static mut PAGE_ALLOCATOR_ORDER_04: [Option<Node>; 1 << 16] = [None; 1 << 16];
pub(super) static mut PAGE_ALLOCATOR_ORDER_05: [Option<Node>; 1 << 15] = [None; 1 << 15];
pub(super) static mut PAGE_ALLOCATOR_ORDER_06: [Option<Node>; 1 << 14] = [None; 1 << 14];
pub(super) static mut PAGE_ALLOCATOR_ORDER_07: [Option<Node>; 1 << 13] = [None; 1 << 13];
pub(super) static mut PAGE_ALLOCATOR_ORDER_08: [Option<Node>; 1 << 12] = [None; 1 << 12];
pub(super) static mut PAGE_ALLOCATOR_ORDER_09: [Option<Node>; 1 << 11] = [None; 1 << 11];
pub(super) static mut PAGE_ALLOCATOR_ORDER_10: [Option<Node>; 1 << 10] = [None; 1 << 10];
pub(super) static mut PAGE_ALLOCATOR_ORDER_11: [Option<Node>; 1 << 9] = [None; 1 << 9];
pub(super) static mut PAGE_ALLOCATOR_ORDER_12: [Option<Node>; 1 << 8] = [None; 1 << 8];
pub(super) static mut PAGE_ALLOCATOR_ORDER_13: [Option<Node>; 1 << 7] = [None; 1 << 7];
pub(super) static mut PAGE_ALLOCATOR_ORDER_14: [Option<Node>; 1 << 6] = [None; 1 << 6];
pub(super) static mut PAGE_ALLOCATOR_ORDER_15: [Option<Node>; 1 << 5] = [None; 1 << 5];
pub(super) static mut PAGE_ALLOCATOR_ORDER_16: [Option<Node>; 1 << 4] = [None; 1 << 4];
pub(super) static mut PAGE_ALLOCATOR_ORDER_17: [Option<Node>; 1 << 3] = [None; 1 << 3];
pub(super) static mut PAGE_ALLOCATOR_ORDER_18: [Option<Node>; 1 << 2] = [None; 1 << 2];

/// Order 19 is the top of the buddy tree: two slots representing the
/// full 4 GiB block in 2 chunks. It is 2 nodes instead of one because
/// otherwise calculations in the [PageAllocator] would overflow and
/// cause division by zero exceptions.
pub(super) static mut PAGE_ALLOCATOR_ORDER_19: [Option<Node>; 1 << 1] = [const { Node::new(None, Some(1)) }, const { Node::new(Some(0), None) }];

/// The global page allocator, fully initialized at compile time.
///
/// - `orders[i]` borrows `PAGE_ALLOCATOR_ORDER_{i:02}` as a `&'static mut` slice. In const context
///   you can't write `&mut PAGE_ALLOCATOR_ORDER_00` directly, but `&mut
///   *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_00)` is allowed and yields the same `&'static
///   mut [Option<Node>]`.
/// - `orders_head[20] = Some(0)` points at the single pre-seeded order-20 free block; every other
///   order starts empty.
pub(super) static PAGE_ALLOCATOR: KernelMutex<PageAllocator<'static>> = KernelMutex::new(PageAllocator::new(
    // Safety:
    // We make sure that this is the only time we take a
    // reference to these arrays so that we only have
    // exlusive access to them in the [PageAllocator]
    unsafe {
        [
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_00),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_01),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_02),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_03),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_04),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_05),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_06),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_07),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_08),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_09),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_10),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_11),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_12),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_13),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_14),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_15),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_16),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_17),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_18),
            &mut *core::ptr::addr_of_mut!(PAGE_ALLOCATOR_ORDER_19),
        ]
    },
    {
        let mut h = [None; ORDERS];
        h[ORDERS - 1] = Some(0);
        h
    },
));
