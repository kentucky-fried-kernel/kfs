const MAX_OBJECTS_PER_CACHE: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub struct BitMap {
    bits: [u8; MAX_OBJECTS_PER_CACHE / 8],
}

impl BitMap {
    pub const fn new() -> Self {
        Self {
            bits: [0u8; MAX_OBJECTS_PER_CACHE / 8],
        }
    }

    #[inline]
    pub const fn set(&mut self, index: usize) {
        self.bits[index / 8] |= 1 << (index % 8)
    }

    #[inline]
    pub const fn unset(&mut self, index: usize) {
        self.bits[index / 8] &= !(1 << (index % 8))
    }
}

impl IntoIterator for BitMap {
    type IntoIter = BitMapIntoIterator;
    type Item = u8;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { bitmap: self, index: 0 }
    }
}

pub struct BitMapIntoIterator {
    bitmap: BitMap,
    index: usize,
}

impl Iterator for BitMapIntoIterator {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index * 8 >= MAX_OBJECTS_PER_CACHE {
            return None;
        }

        let res = (self.bitmap.bits[self.index / 8] >> (self.index % 8)) & 1;
        self.index += 1;

        Some(res)
    }
}
