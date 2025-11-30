// where [(); N / 8]: => An array of size N / 8 must be constructible
// TODO: use 2 bits per node, allows to traverse the tree the following way:
// - 11: all nodes below are allocated, prune subtree
// - 00: all nodes below are free, prefer if choice is possible
// - 10/01: mixed subtree, if no 00 try both

#[derive(Clone, Copy, Debug)]
pub struct BitMap<const N: usize>
where
    [(); N / 8]:,
{
    bits: [u8; N / 8],
}

impl<const N: usize> BitMap<N>
where
    [(); N / 8]:,
{
    pub const fn new() -> Self
    where
        [(); N / 8]:,
    {
        Self { bits: [0u8; N / 8] }
    }

    #[inline]
    pub const fn get(&self, index: usize) -> u8
    where
        [(); N / 8]:,
    {
        (self.bits[index / 8] >> (index % 8)) & 1
    }

    #[inline]
    pub const fn set(&mut self, index: usize)
    where
        [(); N / 8]:,
    {
        self.bits[index / 8] |= 1 << (index % 8);
    }

    #[inline]
    pub const fn unset(&mut self, index: usize) {
        self.bits[index / 8] &= !(1 << (index % 8));
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self as *const BitMap<N> as *const u8
    }
}

impl<const N: usize> IntoIterator for BitMap<N>
where
    [(); N / 8]:,
{
    type IntoIter = BitMapIntoIterator<N>;
    type Item = u8;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { bitmap: self, index: 0 }
    }
}

pub struct BitMapIntoIterator<const N: usize>
where
    [(); N / 8]:,
{
    bitmap: BitMap<N>,
    index: usize,
}

impl<const N: usize> Iterator for BitMapIntoIterator<N>
where
    [(); N / 8]:,
{
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index * 8 >= N {
            return None;
        }

        let res = (self.bitmap.bits[self.index / 8] >> (self.index % 8)) & 1;
        self.index += 1;

        Some(res)
    }
}
