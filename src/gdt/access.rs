#[derive(PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Presence {
    Invalid = 0,
    Valid = 1,
}

#[derive(PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum DescriptorPrivilege {
    Lvl0 = 0,
    Lvl1 = 1,
    Lvl2 = 2,
    Lvl3 = 3,
}
#[derive(PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SegmentType {
    System = 0,
    CodeOrData = 1,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum ExecutabilityType {
    Data = 0,
    Code = 1,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    GrowsUp = 0,
    GrowsDown = 1,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum ReadWriteAble {
    Clear = 0,
    Set = 1,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum AccessBit {
    OnlyForSpecial = 0,
    Default = 1,
}

/// The access byte is part of a Global Descriptor Table entry and represents
/// its permissions. This is required for the CPU to know whether, how, and by who
/// the entry is accessible.
pub struct Access {
    p: Presence,
    dpl: DescriptorPrivilege,
    s: SegmentType,
    e: ExecutabilityType,
    dc: Direction,
    rw: ReadWriteAble,
    a: AccessBit,
}

impl Access {
    pub fn new(
        p: Presence,
        dpl: DescriptorPrivilege,
        s: SegmentType,
        e: ExecutabilityType,
        dc: Direction,
        rw: ReadWriteAble,
        a: AccessBit,
    ) -> Self {
        Access {
            p,
            dpl,
            s,
            e,
            dc,
            rw,
            a,
        }
    }

    pub fn to_u8(&self) -> u8 {
        let mut result: u8 = 0;

        result |= (self.p as u8) << 7;
        result |= (self.dpl as u8) << 5;
        result |= (self.s as u8) << 4;
        result |= (self.e as u8) << 3;
        result |= (self.dc as u8) << 2;
        result |= (self.rw as u8) << 1;
        result |= (self.a as u8);

        result
    }
}
