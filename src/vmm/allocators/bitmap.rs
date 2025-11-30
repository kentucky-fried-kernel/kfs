// where [(); N / G]: => An array of size N / G must be constructible
// TODO: use 2 bits per node, allows to traverse the tree the following way:
// - 11: all nodes below are allocated, prune subtree
// - 00: all nodes below are free, prefer if choice is possible
// - 10/01: mixed subtree, if no 00 try both

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuddyAllocatorNode {
    Free = 0b00,
    PartiallyAllocated = 0b10,
    FullyAllocated = 0b11,
}

impl core::fmt::Display for BuddyAllocatorNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("0b{:02b}", u8::from(self)))
    }
}

impl const From<u8> for BuddyAllocatorNode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Self::Free,
            0b10 => Self::PartiallyAllocated,
            0b11 => Self::FullyAllocated,
            _ => unreachable!(),
        }
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

/// `N` (number): Total number of entries
/// `G` (granularity): Entries per byte
#[derive(Clone, Copy, Debug)]
pub struct BitMap<const N: usize, const G: usize>
where
    [(); N / G]:,
{
    bits: [u8; N / G],
}

impl<const N: usize, const G: usize> BitMap<N, G>
where
    [(); N / G]:,
{
    const MASK: u8 = (1 << (8 / G)) - 1;

    pub const fn new() -> Self
    where
        [(); N / G]:,
    {
        Self { bits: [0u8; N / G] }
    }

    #[inline]
    pub const fn get(&self, index: usize) -> u8
    where
        [(); N / G]:,
    {
        ((self.bits[index / G] >> (index % G)) & Self::MASK).into()
    }

    #[inline]
    pub const fn set(&mut self, index: usize, value: u8)
    where
        [(); N / G]:,
    {
        self.clear(index);
        self.bits[index / G] |= (value & Self::MASK) << (index % G);
    }

    #[inline]
    pub const fn clear(&mut self, index: usize) {
        self.bits[index / G] &= !(Self::MASK << (index % G));
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self as *const BitMap<N, G> as *const u8
    }
}

impl<const N: usize, const G: usize> IntoIterator for BitMap<N, G>
where
    [(); N / G]:,
{
    type IntoIter = BitMapIntoIterator<N, G>;
    type Item = u8;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { bitmap: self, index: 0 }
    }
}

pub struct BitMapIntoIterator<const N: usize, const G: usize>
where
    [(); N / G]:,
{
    bitmap: BitMap<N, G>,
    index: usize,
}

impl<const N: usize, const G: usize> Iterator for BitMapIntoIterator<N, G>
where
    [(); N / G]:,
{
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index * 8 >= N {
            return None;
        }

        let res = (self.bitmap.bits[self.index / G] >> (self.index % G)) & 0b11;
        self.index += 1;

        Some(res)
    }
}
