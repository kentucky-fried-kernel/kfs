//! Physical-address-space page allocator.
//!
//! This module implements the [`PageAllocator`] used by the VMM to hand out
//! page-aligned regions of the 32-bit physical address space. It is a
//! **buddy allocator** with page (4 KiB) granularity and a maximum span of
//! 4 GiB.
//!
//! # Design overview
//!
//! The 4 GiB address space is modeled as a binary tree of power-of-two
//! blocks. Each *level* of the tree is called an **order**:
//!
//! - Order 0 is the finest granularity: one block = 1 page = 4 KiB.
//! - Order `o` has blocks of `2^o` pages (`2^o * PAGE_SIZE` bytes).
//! - Order 19 is the top: blocks of `2^19 * 4 KiB = 2 GiB`, two of them span the whole 4 GiB range.
//!
//! At each order, free blocks are linked in a **doubly-linked free list**.
//! The list nodes are stored *in-place* in a pre-allocated array
//! (one array per order, see [`crate::arch::x86::vmm::state`]); the slot
//! at index `i` in order `o`'s array describes the block starting at page
//! index `i * 2^o`.
//!
//! A slot's state is:
//! - `None`: the block is either currently allocated, or has been merged into a larger free block
//!   at a higher order.
//! - `Some(Node)`: the block is free, and `Node` encodes its position in that order's free list
//!   ([`Node::prev`] / [`Node::next`]).
//!
//! # Allocation
//!
//! [`PageAllocator::alloc`] finds the smallest order that fits the request,
//! tries to pop a block from that order's free list, and if empty splits a
//! block from a higher order down to the requested size. Splitting a block
//! at order `o` removes it from `o`'s free list and adds its two halves to
//! `o - 1`'s free list. See [`PageAllocator::split`] and
//! [`PageAllocator::split_node`].
//!
//! [`PageAllocator::alloc_at`] allocates a specific address range. It is
//! primarily used during boot to mark reserved regions (BIOS, ACPI, the
//! kernel image itself) as allocated.
//!
//! # Freeing
//!
//! [`PageAllocator::dealloc`] adds the block back to the appropriate
//! order's free list. Buddy merging does **not** happen eagerly on
//! `dealloc`; it is performed lazily by [`PageAllocator::coalesce`], which
//! is called from the `alloc` slow path when a request cannot be served
//! from the existing free lists.
//!
//! # Complexity
//!
//! - `alloc` (hot path, block available at requested order): `O(1)`.
//! - `alloc` (slow path, splitting required): `O(log N)`.
//! - `alloc` (after coalesce): `O(N)` in the number of pages, since coalesce walks every order's
//!   backing array.
//! - `dealloc`: `O(1)`.
//! - `alloc_at`: `O(N)` because it always calls [`coalesce`] first, to guarantee the target range
//!   is represented at a single order before splitting down to it.
//!
//! # Invariants
//!
//! The following invariants must hold between any two public calls on a
//! `PageAllocator`:
//!
//! 1. A block at order `o`, index `i` is free (`Some`) **if and only if** neither its parent block
//!    at order `o + 1` (index `i / 2`) nor any of its descendants at lower orders are free.
//! 2. `orders_head[o]` is `Some(i)` if and only if order `o`'s free list is non-empty, and `i` is
//!    the head of that list.
//! 3. For every `Some(node)` at order `o`, following `node.next()` / `node.prev()` yields a
//!    well-formed doubly-linked list terminated by `None` at both ends.
//!
//! Invariant (1) is what allows us to skip eager merging on `dealloc`: a
//! freshly-freed block can't have its buddy already merged upward, because
//! that would mean the buddy's `Some` state at the higher order implied
//! this block was *also* free, contradicting the fact that it was just
//! allocated.

use crate::{printkln, serial_println};

use super::page::PAGE_SIZE;
use core::num::NonZeroU64;

/// A packed free-list link.
///
/// Each `Node` encodes a previous-index and a next-index into its order's
/// backing array, using a single [`NonZeroU64`]. The layout is:
///
/// ```text
///   bit 63          : always 1 (so the `u64` is never zero, enabling the
///                     `Option<Node>` niche optimization)
///   bits 62..=42    : unused (reserved)
///   bits 41..=21    : encoded `prev` (21 bits)
///   bits 20..=0     : encoded `next` (21 bits)
/// ```
///
/// Each 21-bit field stores an `Option<usize>` where indices in
/// `0..=(1 << 20) - 1` are stored as `index + 1`, and `None` is stored as
/// `0`. This lets us pack `Option<usize>` into 21 bits without a separate
/// tag byte, while leaving room for the full 2^20 page indices used by
/// order 0.
///
/// The `Option<Node>` layout takes exactly 8 bytes thanks to bit 63 always
/// being set: `None` is represented by the all-zero bit pattern, so the
/// backing arrays live in `.bss` and cost no ROM space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Node(NonZeroU64);

impl Node {
    /// The largest value representable in a 20-bit index field.
    ///
    /// Order 0 has exactly `1 << 20 = 2^20` slots, so indices range
    /// `0..=U20_MAX`. Attempting to [`encode`](Self::encode) a value
    /// greater than this returns `None`.
    const U20_MAX: u32 = (1 << 20) - 1;

    /// Constructs a new `Node` from optional `prev` and `next` indices.
    ///
    /// Returns `None` if either `prev` or `next` is `Some(v)` with
    /// `v > U20_MAX`, which cannot be represented in 21 bits. Succeeds
    /// with `Some(..)` for any valid combination, including
    /// `(None, None)` — an isolated node with no neighbors.
    ///
    /// This function is `const` so it can be used in static initializers
    /// (see [`crate::arch::x86::vmm::state::PAGE_ALLOCATOR_ORDER_19`]).
    pub const fn new(prev: Option<usize>, next: Option<usize>) -> Option<Self> {
        let p = Self::encode(prev)?;
        let n = Self::encode(next)?;

        let raw = (1u64 << 63) | (p << 21) | n;
        NonZeroU64::new(raw).map(Self)
    }

    /// Encodes an optional index into the 21-bit wire format.
    ///
    /// - `None` → `0`
    /// - `Some(v)` with `v <= U20_MAX` → `v + 1`
    /// - `Some(v)` with `v > U20_MAX` → `None` (caller error)
    const fn encode(val: Option<usize>) -> Option<u64> {
        match val {
            Some(v) if v <= Self::U20_MAX as usize => Some((v + 1) as u64),
            None => Some(0),
            _ => None,
        }
    }

    /// Decodes a 21-bit wire value back into an `Option<usize>`.
    ///
    /// The inverse of [`encode`](Self::encode): `0` → `None`, any other
    /// value `v` → `Some(v - 1)`. Only the low 21 bits of `bits` are
    /// examined; higher bits are ignored, so the caller can pass the raw
    /// `u64` without pre-masking.
    const fn decode(bits: u64) -> Option<usize> {
        let v = (bits & 0x1F_FFFF) as usize;
        if v == 0 { None } else { Some(v - 1) }
    }

    /// Returns the index of the next free block in this order's list, or
    /// `None` if this is the tail.
    fn next(self) -> Option<usize> {
        Self::decode(self.0.get())
    }

    /// Returns the index of the previous free block in this order's list,
    /// or `None` if this is the head.
    fn prev(self) -> Option<usize> {
        Self::decode(self.0.get() >> 21)
    }
}

/// Number of orders in the buddy tree.
///
/// With 4 KiB pages over a 4 GiB address space, the tree has orders
/// `0..=19`:
///
/// - order 0: `2^20` slots × 1 page   = 4 GiB
/// - order 1: `2^19` slots × 2 pages  = 4 GiB
/// - ...
/// - order 19: 2 slots × `2^19` pages = 4 GiB
///
/// Order 19 is the top because indexing into a hypothetical order 20
/// would overflow the index-arithmetic used throughout the allocator.
pub(super) const ORDERS: usize = 20;

/// A buddy-style allocator over a 4 GiB virtual address space with page
/// (4 KiB) granularity.
///
/// See the [module-level documentation](self) for the big picture.
///
/// # Lifetime
///
/// `PageAllocator<'a>` borrows its backing storage (one `&'a mut [Option<Node>]`
/// per order) for lifetime `'a`. In practice `'a = 'static`, because the
/// backing arrays are `static mut` globals defined in
/// [`crate::arch::x86::vmm::state`].
#[derive(Debug)]
pub(super) struct PageAllocator<'a> {
    /// Per-order backing storage. `orders[o]` has `2^(20 - o)` entries
    /// except for `o == 19`, which has 2 entries so that the top of the
    /// tree can be represented without overflowing page-count arithmetic.
    orders: [&'a mut [Option<Node>]; ORDERS],

    /// Head of each order's free list. `orders_head[o] == Some(i)`
    /// means `orders[o][i]` is `Some(..)` with `prev == None`, and is
    /// the first element of order `o`'s free list.
    orders_head: [Option<usize>; ORDERS],
}

impl<'a> PageAllocator<'a> {
    /// Builds a `PageAllocator` from pre-allocated per-order storage.
    ///
    /// Used exclusively from [`crate::arch::x86::vmm::state`] at compile
    /// time to construct the global [`PAGE_ALLOCATOR`] static. The
    /// `orders` slices must have the correct per-order length
    /// (`2^(20 - i)` entries for order `i`, with the top order having 2
    /// slots); this is not checked at runtime.
    ///
    /// `orders_head` should describe whatever free blocks are seeded in
    /// `orders`. For the default initializer this is
    /// `orders_head[ORDERS - 1] = Some(0)` and `None` elsewhere, meaning
    /// the entire 4 GiB is one free block at the top of the tree.
    ///
    /// [`PAGE_ALLOCATOR`]: crate::arch::x86::vmm::state::PAGE_ALLOCATOR
    pub const fn new(orders: [&'a mut [Option<Node>]; ORDERS], orders_head: [Option<usize>; ORDERS]) -> Self {
        Self { orders, orders_head }
    }

    /// Finds the smallest order whose block size covers `size` bytes
    /// *starting from `addr`*, accounting for the alignment waste
    /// introduced when `addr` is not aligned to that order's natural
    /// boundary.
    ///
    /// A block at order `o` has size `2^o * PAGE_SIZE` and must start at
    /// an address that is a multiple of that size. If `addr` is offset
    /// from such a boundary by `k` bytes, then the block that contains
    /// `addr` only has `2^o * PAGE_SIZE - k` usable bytes at `addr` — so
    /// we must round up accordingly before comparing against the order's
    /// capacity.
    ///
    /// Returns `None` if no order in `0..ORDERS` can cover the request.
    fn find_smallest_order_fit_at_addr(addr: usize, size: usize) -> Option<usize> {
        for o in 0..ORDERS {
            let order_size_pages = pow2(o);

            let alignment_offset = addr % (order_size_pages * PAGE_SIZE);
            let size_with_alignment = size + alignment_offset;
            let pages = (size_with_alignment + PAGE_SIZE - 1) / PAGE_SIZE;

            if pages <= order_size_pages {
                return Some(o);
            }
        }

        None
    }

    /// Finds the smallest order whose block size fits `pages` pages.
    ///
    /// Unlike [`find_smallest_order_fit_at_addr`], this does not account
    /// for alignment — it assumes the caller doesn't care where the
    /// block lives. Used by [`alloc`](Self::alloc).
    fn find_smallest_order_fit(pages: usize) -> Option<usize> {
        for o in 0..ORDERS {
            let order_size_pages = pow2(o);

            if pages <= order_size_pages {
                return Some(o);
            }
        }

        None
    }

    /// Pops the head block off `order`'s free list and returns its base
    /// address, or `None` if the list is empty.
    ///
    /// Maintains invariant (2) and (3) from the [module docs](self) by
    /// delegating the list bookkeeping to [`node_remove`](Self::node_remove).
    fn node_drain(&mut self, order: usize) -> Option<*mut u8> {
        let next = self.orders_head[order]?;

        self.node_remove(next, order);

        let addr = next * pow2(order) * PAGE_SIZE;

        Some(addr as *mut u8)
    }

    /// Allocates a contiguous region of at least `size` bytes.
    ///
    /// The returned pointer is page-aligned and points to an unused
    /// block of at least `2^o * PAGE_SIZE` bytes, where `o` is the
    /// smallest order whose block size fits `size`. The caller is
    /// responsible for tracking the actual allocated size (typically by
    /// rounding `size` up to the next power of two in pages) so it can
    /// be passed back to [`dealloc`](Self::dealloc) correctly.
    ///
    /// # Algorithm
    ///
    /// 1. Pop from the target order's free list. Fast path, `O(1)`.
    /// 2. If empty, split a larger block down. `O(log N)`.
    /// 3. If that fails, run [`coalesce`](Self::coalesce) to merge any outstanding buddy pairs,
    ///    then retry steps 1–2. `O(N)`.
    ///
    /// Returns `None` only when the entire address space is exhausted or
    /// so fragmented that no run of `size` contiguous free bytes exists
    /// at any order.
    ///
    /// # Panics
    ///
    /// - If `size` is 0.
    /// - If `size` exceeds 2 GiB (the largest request the top-most callable order, 19, can service
    ///   through this entry point).
    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let two_gbs = 1 << 31;
        assert!(size <= two_gbs);
        assert!(size != 0);

        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

        let smallest_order = Self::find_smallest_order_fit(pages)?;

        // Fast path: a block already exists at the requested order.
        if let Some(addr) = self.node_drain(smallest_order) {
            return Some(addr);
        }

        // Slow path: walk up and split a larger block down to our size.
        if smallest_order + 1 < ORDERS {
            self.split(smallest_order + 1);
        }
        if let Some(addr) = self.node_drain(smallest_order) {
            return Some(addr);
        }

        // Slower path: nothing large enough is available in one piece.
        // Coalesce outstanding buddy pairs to rebuild larger blocks,
        // then try splitting again.
        self.coalesce();

        if let Some(addr) = self.node_drain(smallest_order) {
            return Some(addr);
        }
        if smallest_order + 1 < ORDERS {
            self.split(smallest_order + 1);
        }
        if let Some(addr) = self.node_drain(smallest_order) {
            return Some(addr);
        }

        // Address space exhausted.
        None
    }

    /// Allocates the specific range `[ptr, ptr + size)`, marking it as
    /// used. Returns `Some(ptr)` on success, `None` if the range could
    /// not be reserved (e.g., already allocated).
    ///
    /// Primarily used during boot to mark pre-existing reserved regions
    /// (kernel image, BIOS/ACPI data, etc.) as allocated before the
    /// allocator starts serving general requests.
    ///
    /// # Algorithm
    ///
    /// 1. Find the smallest order whose block size covers `size` *starting at `ptr`*, accounting
    ///    for alignment waste.
    /// 2. Fully [`coalesce`](Self::coalesce) so that the block containing `ptr` is represented at
    ///    exactly one order — namely, the highest one for which it remains free. This step is
    ///    `O(N)`, which is acceptable because `alloc_at` is only called during boot.
    /// 3. Walk down from that order, splitting blocks containing `ptr` at each level, until we
    ///    reach the target order.
    /// 4. Remove the now-isolated block at the target order.
    ///
    /// # Panics
    ///
    /// If `size` is 0.
    pub fn alloc_at(&mut self, ptr: *mut u8, size: usize) -> Option<*mut u8> {
        assert!(size != 0);

        let smallest_order = Self::find_smallest_order_fit_at_addr(ptr as usize, size)?;

        // Ensure the block containing `ptr` is represented at a single
        // order before we try to split down to it.
        self.coalesce();

        // Split down from the top toward the target order, always
        // splitting the block containing `ptr`.
        for o in (smallest_order + 1..(ORDERS)).rev() {
            let entry_size = pow2(o) * PAGE_SIZE;

            let index_node = ptr as usize / entry_size;

            let _ = self.split_node(index_node, o);
        }

        let entry_size = pow2(smallest_order) * PAGE_SIZE;
        let order_index = ptr as usize / entry_size;

        if self.orders[smallest_order][order_index].is_some() {
            self.node_remove(order_index, smallest_order);
            return Some(ptr);
        }

        None
    }

    /// Returns the `size` bytes starting at `ptr` to the free list.
    ///
    /// The block is added back at the smallest order whose block size
    /// covers `[ptr, ptr + size)` with `ptr`'s alignment. Buddy merging
    /// is **not** performed here; it happens lazily the next time
    /// [`alloc`](Self::alloc) needs a block larger than what's
    /// currently free.
    ///
    /// If `ptr`/`size` don't correspond to any representable order
    /// (e.g., `size` is zero or huge), this is a no-op rather than a
    /// panic, since `dealloc` is typically called from a `Drop` impl
    /// where panicking would abort.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `(ptr, size)` corresponds to a
    /// previous successful [`alloc`](Self::alloc) or
    /// [`alloc_at`](Self::alloc_at) that has not already been freed.
    /// Double-free or freeing a region that was never allocated will
    /// corrupt the allocator state (see invariant (1) in the
    /// [module docs](self)).
    pub fn dealloc(&mut self, ptr: *mut u8, size: usize) {
        let order = match Self::find_smallest_order_fit_at_addr(ptr as usize, size) {
            Some(o) => o,
            None => return,
        };

        let index_node = ptr as usize / (pow2(order) * PAGE_SIZE);

        self.node_add(index_node, order);
    }

    /// Merges every pair of free buddies upward, bottom-up.
    ///
    /// For each order from 0 up to `ORDERS - 2`, scans every
    /// buddy pair `(2k, 2k + 1)`. If both buddies are free, removes them
    /// from the current order's free list and inserts their parent
    /// (index `k` at order `o + 1`) into the next order's free list.
    ///
    /// This is `O(N)` in the total number of tree slots, so it should
    /// only be called when the alternative (failing an allocation) is
    /// worse: from [`alloc`](Self::alloc)'s slow path, and always from
    /// [`alloc_at`](Self::alloc_at) during boot.
    ///
    /// # Correctness note
    ///
    /// After `coalesce`, the invariants of the free lists still hold.
    /// A merged pair's parent is guaranteed to land in a slot that
    /// was `None` before the merge — because if the parent had been
    /// `Some`, invariant (1) would have required the two children to
    /// be `None`, contradicting the fact that we just observed both as
    /// free.
    pub fn coalesce(&mut self) {
        for index_order in 0..(ORDERS - 1) {
            for index_node in (0..(pow2(ORDERS - 1 - index_order))).step_by(2) {
                let left = self.orders[index_order][index_node];
                let right = self.orders[index_order][index_node + 1];
                if left.is_some() && right.is_some() {
                    self.node_remove(index_node, index_order);
                    self.node_remove(index_node + 1, index_order);
                    self.node_add(index_node / 2, index_order + 1);
                }
            }
        }
    }

    /// Splits a free block at `order` into two buddies at `order - 1`.
    ///
    /// If `order`'s free list is empty, recurses upward to create a
    /// block at `order` first by splitting from `order + 1`. This
    /// bottoms out at `ORDERS - 1`; if the top of the tree is also
    /// empty, the function is a no-op.
    ///
    /// Called from [`alloc`](Self::alloc)'s slow path to manufacture a
    /// block at the requested order when no block of that size is
    /// currently free but some larger block is.
    fn split(&mut self, order: usize) {
        match self.orders_head[order] {
            Some(next) => {
                let _ = self.split_node(next, order);
                return;
            }
            None => {
                if order < (ORDERS - 1) {
                    self.split(order + 1);
                }
            }
        }

        if let Some(next) = self.orders_head[order] {
            let _ = self.split_node(next, order);
        }
    }

    /// Splits a specific free block at `(index_order, index_node)` into
    /// its two buddies at `index_order - 1`.
    ///
    /// Returns `Err(())` if the slot is not currently free (callers
    /// that don't care — namely [`split`](Self::split) and the loop in
    /// [`alloc_at`](Self::alloc_at) — discard the result).
    ///
    /// # Panics
    ///
    /// If `index_order == 0`, since there is no order below 0 to split
    /// into.
    fn split_node(&mut self, index_node: usize, index_order: usize) -> Result<(), ()> {
        assert!(index_order != 0);
        if self.orders[index_order][index_node].is_none() {
            return Err(());
        }

        self.node_remove(index_node, index_order);
        self.node_add((index_node * 2) + 1, index_order - 1);
        self.node_add(index_node * 2, index_order - 1);
        Ok(())
    }

    /// Inserts a free block at the head of its order's free list.
    ///
    /// Assumes `orders[index_order][index_node]` is currently `None`;
    /// calling this on an already-free slot overwrites the existing
    /// node and corrupts the list. Invariant (1) from the
    /// [module docs](self) is what guarantees this precondition holds
    /// for every internal caller: a block can only transition from
    /// allocated/coalesced (`None`) to free (`Some`) via `node_add`,
    /// never from one `Some` state to another.
    fn node_add(&mut self, index_node: usize, index_order: usize) {
        let node = Node::new(None, self.orders_head[index_order]);

        if let Some(prev_index) = self.orders_head[index_order] {
            let prev = &mut self.orders[index_order][prev_index];
            *prev = Node::new(Some(index_node), prev.unwrap().next());
        }

        self.orders[index_order][index_node] = node;

        self.orders_head[index_order] = Some(index_node);
    }

    /// Removes a specific free block from its order's free list.
    ///
    /// Unlinks `orders[index_order][index_node]` from both of its
    /// neighbors (or updates `orders_head` if it was the head) and
    /// clears the slot to `None`.
    ///
    /// # Panics
    ///
    /// If the slot is already `None` — there is nothing to remove.
    /// Internal callers always check this beforehand.
    fn node_remove(&mut self, index_node: usize, index_order: usize) {
        let node = self.orders[index_order][index_node].take();
        let node = node.unwrap();
        match node.prev() {
            Some(index) => {
                let prev = self.orders[index_order][index].unwrap();
                self.orders[index_order][index] = Node::new(prev.prev(), node.next());
            }
            None => {
                self.orders_head[index_order] = node.next();
            }
        }

        if let Some(index) = node.next() {
            let next = self.orders[index_order][index].unwrap();
            self.orders[index_order][index] = Node::new(node.prev(), next.next());
        }
    }

    /// Prints every free block across every order to the serial console.
    ///
    /// Output format:
    /// ```text
    /// order  N | node IIIIII | addr 0xAAAAAAAA | size 0xS (P pages)
    /// ```
    ///
    /// Orders are walked from highest to lowest so that large free
    /// regions appear first. Debug/diagnostic tool only.
    pub fn print(&self) {
        for order in (0..ORDERS).rev() {
            let mut current = self.orders_head[order];
            while let Some(index) = current {
                let addr = index * pow2(order) * PAGE_SIZE;
                let size = pow2(order) * PAGE_SIZE;
                serial_println!(
                    "order {:2} | node {:6} | addr 0x{:08x} | size 0x{:x} ({:x} pages)",
                    order,
                    index,
                    addr,
                    size,
                    pow2(order)
                );
                let node = self.orders[order][index].unwrap();
                current = node.next();
            }
        }
    }

    /// Prints the total number of free bytes across every order as a
    /// human-readable `MB + KB` figure to both the kernel log
    /// ([`printkln!`]) and the serial console ([`serial_println!`]).
    ///
    /// Used once at the end of boot to report how much RAM the kernel
    /// successfully claimed after marking reserved regions.
    pub fn print_free(&self) {
        let mut total_bytes: u64 = 0;

        for order in 0..ORDERS {
            let block_bytes = (pow2(order) as u64) * (PAGE_SIZE as u64);
            let count = self.count_free_at(order) as u64;
            total_bytes += count * block_bytes;
        }

        let mb = total_bytes / (1024 * 1024);
        let kb = (total_bytes % (1024 * 1024)) / 1024;

        printkln!("Memory available after boot: {} MB {} KB free", mb, kb);
        serial_println!("Memory available after boot: {} MB {} KB free", mb, kb);
    }

    /// Walks `order`'s free list and returns the number of free blocks.
    ///
    /// `O(k)` where `k` is the number of free blocks at that order.
    /// Used only by [`print_free`](Self::print_free).
    fn count_free_at(&self, order: usize) -> usize {
        let mut count = 0;
        let mut current = self.orders_head[order];

        while let Some(idx) = current {
            count += 1;
            current = self.orders[order][idx].expect("free list index points to empty slot").next();
        }

        count
    }
}

/// Returns `2^power` as a `usize`.
///
/// # Panics
///
/// If `power >= ORDERS`. This guard ensures that callers who then
/// multiply the result by [`PAGE_SIZE`] cannot overflow `usize` on
/// 32-bit targets: the maximum returned value is `2^(ORDERS - 1) =
/// 2^19`, and `2^19 * 4096 = 2^31` fits in a 32-bit `usize`.
fn pow2(power: usize) -> usize {
    assert!(power < ORDERS);
    2u32.pow(power as u32) as usize
}
