// where [(); N / G]: => An array of size N / G must be constructible
// TODO: use 2 bits per node, allows to traverse the tree the following way:
// - 11: all nodes below are allocated, prune subtree
// - 00: all nodes below are free, prefer if choice is possible
// - 10/01: mixed subtree, if no 00 try both

/// `N` (number): Total number of entries
/// `G` (granularity): Entries per byte (one of 1, 2, 4, 8)
#[derive(Clone, Copy, Debug)]
pub struct BitMap<const N: usize, const G: usize>
where
    [(); N / G]:,
{
    bits: [u8; N / G],
}

impl<const N: usize, const G: usize> Default for BitMap<N, G>
where
    [(); N / G]:,
{
    fn default() -> Self
    where
        [(); N / G]:,
    {
        Self::new()
    }
}

impl<const N: usize, const G: usize> BitMap<N, G>
where
    [(); N / G]:,
{
    const BITS_PER_ENTRY: usize = 8 / G;
    const MASK: u8 = (1 << (Self::BITS_PER_ENTRY)) - 1;

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
        (self.bits[index / G] >> ((index % G) * Self::BITS_PER_ENTRY)) & Self::MASK
    }

    #[inline]
    pub const fn set(&mut self, index: usize, value: u8)
    where
        [(); N / G]:,
    {
        self.clear(index);
        self.bits[index / G] |= (value & Self::MASK) << ((index % G) * Self::BITS_PER_ENTRY);
    }

    #[inline]
    pub const fn clear(&mut self, index: usize) {
        self.bits[index / G] &= !(Self::MASK << ((index % G) * Self::BITS_PER_ENTRY));
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
        if self.index >= N {
            return None;
        }

        let res = (self.bitmap.bits[self.index / G] >> ((self.index % G) * BitMap::<N, G>::BITS_PER_ENTRY)) & BitMap::<N, G>::MASK;
        self.index += 1;

        Some(res)
    }
}
