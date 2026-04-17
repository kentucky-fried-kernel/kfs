use crate::serial_println;

use super::page::PAGE_SIZE;
use core::num::NonZeroU64;
use core::panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Node(NonZeroU64);

impl Node {
    const U20_MAX: u32 = (1 << 20) - 1;

    pub const fn new(prev: Option<usize>, next: Option<usize>) -> Option<Self> {
        let p = Self::encode(prev)?;
        let n = Self::encode(next)?;

        let raw = (1u64 << 63) | (p << 21) | n;
        NonZeroU64::new(raw).map(Self)
    }

    const fn encode(val: Option<usize>) -> Option<u64> {
        match val {
            Some(v) if v <= Self::U20_MAX as usize => Some((v + 1) as u64),
            None => Some(0),
            _ => None,
        }
    }

    const fn decode(bits: u64) -> Option<usize> {
        let v = (bits & 0x1F_FFFF) as usize;
        if v == 0 { None } else { Some(v - 1) }
    }

    fn next(self) -> Option<usize> {
        Self::decode(self.0.get())
    }
    fn prev(self) -> Option<usize> {
        Self::decode(self.0.get() >> 21)
    }
}

pub(super) const ORDERS: usize = 21;

#[derive(Debug)]
pub(super) struct PageAllocator<'a> {
    orders: [&'a mut [Option<Node>]; ORDERS],
    orders_head: [Option<usize>; ORDERS],
}

impl<'a> PageAllocator<'a> {
    pub const fn new(orders: [&'a mut [Option<Node>]; ORDERS], orders_head: [Option<usize>; ORDERS]) -> Self {
        Self { orders, orders_head }
    }
    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }
        let mut pages = size / PAGE_SIZE;
        if size % PAGE_SIZE != 0 {
            pages += 1;
        }

        let mut smallest_order = None;

        for o in 0..ORDERS {
            let order_size = pow2(o);

            if pages <= order_size {
                smallest_order = Some(o);
                break;
            }
        }

        let smallest_order = smallest_order?;

        // Block already exists in the requested order
        if let Some(next) = self.orders_head[smallest_order] {
            self.node_remove(next, smallest_order);

            let res = next * pow2(smallest_order) * PAGE_SIZE;

            return Some(res as *mut u8);
        }

        // It searches if higher order blocks can be broken up for the requestsed order
        if smallest_order + 1 < ORDERS {
            self.split(smallest_order + 1);
        }

        if let Some(next) = self.orders_head[smallest_order] {
            self.node_remove(next, smallest_order);

            let res = next * pow2(smallest_order) * PAGE_SIZE;

            return Some(res as *mut u8);
        }

        // Try to coalesce blocks to build requested order from smaller order blocks
        self.coalesce();

        // Block already exists in the requested order
        if let Some(next) = self.orders_head[smallest_order] {
            self.node_remove(next, smallest_order);

            let res = next * pow2(smallest_order) * PAGE_SIZE;

            return Some(res as *mut u8);
        }

        // It searches if higher order blocks can be broken up for the requestsed order
        if smallest_order + 1 < ORDERS {
            self.split(smallest_order + 1);
        }

        if let Some(next) = self.orders_head[smallest_order] {
            self.node_remove(next, smallest_order);

            let res = next * pow2(smallest_order) * PAGE_SIZE;

            return Some(res as *mut u8);
        }
        None
    }

    pub fn alloc_at(&mut self, ptr: *mut u8, size: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }
        let mut smallest_order = None;

        for o in 0..ORDERS {
            let order_size = pow2(o);

            let mut pages = (size + ptr as usize % (order_size * PAGE_SIZE)) / PAGE_SIZE;
            if size % PAGE_SIZE != 0 {
                pages += 1;
            }
            if pages <= order_size {
                smallest_order = Some(o);
                break;
            }
        }

        let smallest_order = smallest_order?;

        if smallest_order + 1 < ORDERS {
            // The top node has to be done manually because calculating the entry size would perfectly
            // overflow into 0 again.
            self.split_node(0, ORDERS - 1);
        }

        for o in (smallest_order + 1..(ORDERS - 1)).rev() {
            let entry_size = pow2(o) * PAGE_SIZE;
            let index_node = ptr as usize / entry_size;

            self.split_node(index_node, o);
        }

        let entry_size = pow2(smallest_order) * PAGE_SIZE;
        if let Some(_) = self.orders[smallest_order][ptr as usize / entry_size] {
            self.node_remove(ptr as usize / entry_size, smallest_order);
            return Some(ptr);
        }

        None
    }
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
    pub fn dealloc(&mut self, ptr: *mut u8, size: usize) {
        let mut order = None;

        for o in 0..ORDERS {
            let order_size = pow2(o);
            if o == ORDERS - 1 {
                self.node_add(0, o);
                return;
            }
            let mut pages = (size as u64 + ptr as u64 % (order_size as u64 * PAGE_SIZE as u64)) as usize / PAGE_SIZE;
            if size % PAGE_SIZE != 0 {
                pages += 1;
            }

            if pages <= order_size {
                order = Some(o);
                break;
            }
        }

        let order = match order {
            Some(o) => o,
            None => return,
        };

        let index_node = ptr as usize / (pow2(order) * PAGE_SIZE);

        self.node_add(index_node, order);
    }

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

    fn split(&mut self, order: usize) {
        match self.orders_head[order] {
            Some(next) => {
                self.split_node(next, order);
                return;
            }
            None => {
                if order < (ORDERS - 1) {
                    self.split(order + 1);
                }
            }
        }

        if let Some(next) = self.orders_head[order] {
            self.split_node(next, order);
        }
    }

    fn split_node(&mut self, index_node: usize, index_order: usize) -> Result<(), ()> {
        assert!(index_order != 0);
        if let None = self.orders[index_order][index_node] {
            return Err(());
        }

        self.node_remove(index_node, index_order);
        self.node_add((index_node * 2) + 1, index_order - 1);
        self.node_add(index_node * 2, index_order - 1);
        Ok(())
    }

    fn node_add(&mut self, index_node: usize, index_order: usize) {
        let node = Node::new(None, self.orders_head[index_order]);

        if let Some(prev_index) = self.orders_head[index_order] {
            let prev = &mut self.orders[index_order][prev_index];
            *prev = Node::new(Some(index_node), prev.unwrap().next());
        }

        self.orders[index_order][index_node] = node;

        self.orders_head[index_order] = Some(index_node);
    }

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

        match node.next() {
            Some(index) => {
                let next = self.orders[index_order][index].unwrap();
                self.orders[index_order][index] = Node::new(node.prev(), next.next());
            }
            None => {}
        }
    }
}

fn pow2(power: usize) -> usize {
    2u32.pow(power as u32) as usize
}
