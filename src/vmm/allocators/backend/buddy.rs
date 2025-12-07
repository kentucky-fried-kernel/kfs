use core::ptr::NonNull;

use crate::{
    bitmap::StaticBitmap,
    expect_opt,
    vmm::{allocators::kmalloc::KfreeError, paging::PAGE_SIZE},
};

pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 25;

pub enum BuddyAllocationError {
    NotEnoughMemory,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuddyAllocatorNode {
    /// This node + all its children are free
    Free = 0b00,

    /// One or more of this node's children are allocated
    PartiallyAllocated = 0b10,

    /// This node and all its children are allocated
    FullyAllocated = 0b11,
}

impl core::fmt::Display for BuddyAllocatorNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("0b{:02b}", u8::from(self)))
    }
}

impl const From<&BuddyAllocatorNode> for u8 {
    fn from(value: &BuddyAllocatorNode) -> Self {
        match value {
            BuddyAllocatorNode::Free => 0b00,
            BuddyAllocatorNode::PartiallyAllocated => 0b10,
            BuddyAllocatorNode::FullyAllocated => 0b11,
        }
    }
}

impl const From<BuddyAllocatorNode> for u8 {
    fn from(value: BuddyAllocatorNode) -> Self {
        value as u8
    }
}

const MAX_BUDDY_ALLOCATOR_LEVEL_INDEX: usize = ((1u64 << 32).ilog2() - 4096u64.ilog2()) as usize;
pub const BUDDY_ALLOCATOR_LEVELS_SIZE: usize = MAX_BUDDY_ALLOCATOR_LEVEL_INDEX + 1;

/// [Buddy Allocator](https://en.wikipedia.org/wiki/Buddy_memory_allocation).
///
/// This allocator manages a block of up to 4GiB with a granularity of 4096B
/// (page size). The tree is represented as an array of bitmaps with 2 bits per
/// node, meaning the total size of the bitmaps referenced by the `levels` array
/// ≈ 524288 bytes `((4GiB / 4096B) / 4) * 2`.
///
/// Since data structure needs to be statically allocated, its size is hardcoded
/// and will waste memory when managing sizes `< 4GiB`. The alternative would be
/// to have a generic `BuddyAllocator` struct, which I may consider at some
/// point but for now it's not that deep.
///
/// A node can have [3 different states][BuddyAllocatorNode]:
/// ```
/// enum BuddyAllocatorNode {
///     Free = 0b00,               // all children are free
///     PartiallyAllocated = 0b10, // some children are allocated
///     FullyAllocated = 0b11,     // all children are allocated
/// }
/// ```
/// When walking the tree looking for free space, this allows for pruning
/// sub-trees that we know will not contain any block of the size we are looking
/// for (which would not be possible with 1-bit nodes).
///
/// Allocation operations are O(log N) on average, O(N) in the worst case
/// (highly fragmented memory, causing a scan of the whole tree).
///
/// Free operations are guaranteed to be O(log N).
///
/// ## Future optimizations
/// We can get rid of the O(N) worst case for allocations by keeping a free list
/// for each level.
/// ```
/// pub struct BuddyAllocator {
///     levels: [&'static mut dyn StaticBitmap; BUDDY_ALLOCATOR_LEVELS_SIZE],
///     free_lists: [Option<NonNull<FreeBlock>>; BUDDY_ALLOCATOR_LEVELS_SIZE],
///     // [...]
/// }
///
/// #[repr(C)]
/// struct FreeBlock {
///     next: Option<NonNull<FreeBlock>>,
///     prev: Option<NonNull<FreeBlock>>,
/// }
///
/// // allocation becomes
/// pub fn alloc(&mut self, size: usize) -> Result<*mut u8, BuddyAllocationError> {
///     if let Some(block) = self.free_lists[get_target_level(size)].take() {
///         self.remove_from_free_list(target_level, block);
///         return Ok(block.as_ptr() as *mut u8);
///     }
///
///     // Split blocks until we find the right size, adding the blocks that we are not
///     // entering to the free list as we go.
/// }
/// ```
pub struct BuddyAllocator {
    /// Stores all possible levels of the bitmap. In order to span 4GiB with
    /// page granularity (where the root block is 4GiB, and the leaf nodes
    /// are 4096B), we need 21 levels (`log2(4294967296) - log2(4096)` gives
    /// us the level index at the smallest granularity, add 1 for the
    /// required size of the array).
    levels: [&'static mut dyn StaticBitmap; BUDDY_ALLOCATOR_LEVELS_SIZE],

    /// Start address of the memory managed by the `BuddyAllocator`.
    root: Option<NonNull<u8>>,

    /// Index of the root bitmap (if `size == 4GiB`, use the full span of the
    /// tree (`root_level = 0`),  if `size == 2GiB`, start one level below
    /// (`root_level = 1`) , and so on).
    root_level: usize,

    /// Size of the block managed by the buddy allocator.
    size: usize,
}

impl BuddyAllocator {
    /// Creates a new `BuddyAllocator` managing `size` bytes starting from
    /// `root`. If `root` cannot be determined at compile-time (which will
    /// be the case if the allocator manages dynamically allocated memory),
    /// `None` can be passed, and `root` can be set later:
    /// ```
    /// // Allocate statically
    /// pub static mut BUDDY_ALLOCATOR: BuddyAllocator = unsafe { BuddyAllocator::new(None, BUDDY_ALLOCATOR_SIZE, LEVELS) };
    ///
    /// // Initialize at runtime
    /// #[allow(static_mut_refs)]
    /// fn init() {
    ///     let ptr = mmap([...], BUDDY_ALLOCATOR_SIZE, [...]);
    ///     unsafe { BUDDY_ALLOCATOR.set_root(NonNull::new(ptr)) };
    /// }
    /// ```
    /// Please note that calling _any_ other function than `set_root` on a
    /// [`BuddyAllocator`] with an uninitialized `root` will
    /// crash the kernel.
    ///
    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined Behavior:
    /// 1. `root`, if `Some(_)`, must point to valid memory with at least `size` reserved bytes.
    /// 2. Since each bitmap size is a different type, we have to resort to dynamic dispatch. Each
    ///    `levels[i]` must actually refer to a `BitMap<{ (1 << i).min(8) }, 4>`, otherwise bad
    ///    things will happen.
    ///
    /// # Panics
    /// This function will panic if passed incorrect arguments, like a `size`
    /// which is not a power of 2, or if `size < 32768 || size >
    /// 4294967296`.
    #[allow(static_mut_refs)]
    pub const unsafe fn new(root: Option<NonNull<u8>>, size: usize, levels: [&'static mut dyn StaticBitmap; BUDDY_ALLOCATOR_LEVELS_SIZE]) -> Self {
        assert!(2usize.pow(size.ilog2()) == size, "size must be a power of 2");
        assert!(size >= 1 << 15, "size must be at least 32768 and at most 4294967296");

        let root_level = 32 - size.ilog2() as usize;

        Self {
            levels,
            root,
            root_level,
            size,
        }
    }

    pub fn set_root(&mut self, root: NonNull<u8>) {
        self.root = Some(root);
    }

    #[inline]
    #[allow(static_mut_refs)]
    fn alloc_internal(&mut self, allocation_size: usize, root: *mut u8, level_block_size: usize, level: usize, index: usize) -> Option<*mut u8> {
        assert!(
            allocation_size.is_multiple_of(PAGE_SIZE),
            "The buddy allocator can only allocate multiples of 0x1000"
        );

        let current_state = self.levels[level].get(index);

        if current_state == BuddyAllocatorNode::FullyAllocated as u8 {
            return None;
        }

        if allocation_size >= level_block_size || level == self.levels.len() {
            if current_state == BuddyAllocatorNode::Free as u8 {
                self.levels[level].set(index, BuddyAllocatorNode::FullyAllocated as u8);
                return Some(root);
            }
            return None;
        }

        let left_child_index = index * 2;
        let allocation = self.alloc_internal(allocation_size, root, level_block_size / 2, level + 1, left_child_index);

        if allocation.is_some() {
            if self.levels[level].get(index) == BuddyAllocatorNode::Free as u8 {
                self.levels[level].set(index, BuddyAllocatorNode::PartiallyAllocated as u8);
            }
            return allocation;
        }

        let right_child_index = index * 2 + 1;
        let allocation = self.alloc_internal(
            allocation_size,
            (root as usize + level_block_size / 2) as *mut u8,
            level_block_size / 2,
            level + 1,
            right_child_index,
        );

        if allocation.is_some() && self.levels[level].get(index) == BuddyAllocatorNode::Free as u8 {
            self.levels[level].set(index, BuddyAllocatorNode::PartiallyAllocated as u8);
        }

        allocation
    }

    /// Allocates a block of memory of size `size` from the buddy allocator,
    /// updating its parents accordingly. Returns [`BuddyAllocationError`]
    /// if not enough memory is available.
    ///
    /// # Panics
    /// This function will panic if passed incorrect parameters, like a `size`
    /// which is not a multiple of `PAGE_SIZE`, or a `size` larger than
    /// `self.size`.
    ///
    /// # Errors
    /// This function will return an error if no fitting memory block is found
    /// for the allocation.
    pub fn alloc(&mut self, size: usize) -> Result<*mut u8, BuddyAllocationError> {
        assert!(size.is_multiple_of(PAGE_SIZE), "The buddy allocator can only allocate multiples of 0x1000");
        assert!(size <= self.size, "The buddy allocator cannot allocate more than its size");
        let root = expect_opt!(self.root, "alloc called on BuddyAllocator without root");

        self.alloc_internal(size, root.as_ptr(), self.size, self.root_level, 0)
            .ok_or(BuddyAllocationError::NotEnoughMemory)
    }

    /// Gets the base index (level 19, page granularity) for a given `addr`.
    /// Used to recurse back from there and find an allocation by address.
    #[inline]
    fn get_base_index(&self, addr: *const u8) -> usize {
        let root = expect_opt!(self.root, "get_base_index called on BuddyAllocator without root");
        assert!(
            (root.as_ptr() as usize..(root.as_ptr() as usize + self.size)).contains(&(addr as usize)),
            "addr is out of range for this allocator"
        );

        (addr as usize - root.as_ptr() as usize) / PAGE_SIZE
    }

    fn update_parent_states(&mut self, level: usize, index: usize) {
        if level == self.root_level {
            return;
        }

        let parent_index = index / 2;

        let left_state = self.levels[level].get(index & !1);
        let right_state = self.levels[level].get(index | 1);

        let parent_state = match (left_state, right_state) {
            (0b00, 0b00) => BuddyAllocatorNode::Free,
            (0b11, 0b11) => BuddyAllocatorNode::FullyAllocated,
            _ => BuddyAllocatorNode::PartiallyAllocated,
        };

        self.levels[level].set(parent_index, parent_state as u8);

        self.update_parent_states(level - 1, parent_index);
    }

    fn coalesce(&mut self, level: usize, index: usize) {
        if level == self.root_level {
            return;
        }

        let buddy_index = if index % 2 == 1 { index - 1 } else { index + 1 };

        let parent_index = index / 2;
        if self.levels[level].get(buddy_index) == BuddyAllocatorNode::Free as u8 {
            self.levels[level - 1].set(parent_index, BuddyAllocatorNode::Free as u8);
            self.coalesce(level - 1, parent_index);
        } else {
            if self.levels[level - 1].get(parent_index) == BuddyAllocatorNode::FullyAllocated as u8 {
                self.levels[level - 1].set(parent_index, BuddyAllocatorNode::PartiallyAllocated as u8);
            }

            if level > self.root_level + 1 {
                self.update_parent_states(level - 1, parent_index);
            }
        }
    }

    /// Frees the memory block pointed to by `addr` and walks the tree
    /// backwards, coalescing the freed block with its parents.
    ///
    /// # Errors
    /// Returns an error when passed a pointer the `BuddyAllocator` does not
    /// own, not sure about this design yet.
    ///
    /// # Panics
    /// This function will panic if passed invalid arguments, like if `addr` is
    /// null, or if the `BuddyAllocator` is not initialized
    /// (`self.root.is_none()`).
    pub fn free(&mut self, addr: *const u8) -> Result<(), KfreeError> {
        assert!(!addr.is_null(), "Cannot free null pointer");
        assert!(self.root.is_some(), "free called on BuddyAllocator without root");

        let mut index = self.get_base_index(addr);

        for level in (self.root_level..self.levels.len()).rev() {
            if self.levels[level].get(index) == BuddyAllocatorNode::FullyAllocated as u8 {
                self.levels[level].set(index, BuddyAllocatorNode::Free as u8);
                self.coalesce(level, index);
                return Ok(());
            }
            index /= 2;
        }

        Err(KfreeError::InvalidPointer)
    }
}
