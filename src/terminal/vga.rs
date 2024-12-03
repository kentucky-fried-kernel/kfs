use core::ptr::{read_volatile, write_volatile};

use super::terminal::Terminal;

pub const VIEW_WIDTH: usize = 80;
pub const VIEW_HEIGHT: usize = 25;
pub const VIEW_BUFFER_SIZE: usize = VIEW_WIDTH * VIEW_HEIGHT;
const VGA_BUFFER_ADDR: *mut u16 = 0xB8000 as *mut u16;

pub fn flush_vga(t: &Terminal) {
    let mut view_offset: usize = 0;
    for (mut index, &entry) in t.buffer.iter().enumerate() {
        index += view_offset;
        if index >= VIEW_BUFFER_SIZE {
            break;
        }

        if (entry & 0xFF) as u8 == b'\n' {
            view_offset += VIEW_WIDTH - (index % VIEW_WIDTH) - 1;
            continue;
        }

        let written_entry = read_entry_from_vga(index).unwrap(); // Have to think about how we want to handle this
        if entry == written_entry {
            continue;
        }

        write_entry_to_vga(index, entry).unwrap(); // Same here have to check if thats fine
    }
}

#[derive(Debug)]
pub struct OutOfBoundsErr;

fn write_entry_to_vga(index: usize, entry: u16) -> Result<(), OutOfBoundsErr> {
    if index >= VIEW_BUFFER_SIZE {
        return Err(OutOfBoundsErr);
    }

    unsafe { write_volatile(VGA_BUFFER_ADDR.add(index), entry) }
    Ok(())
}

fn read_entry_from_vga(index: usize) -> Result<u16, OutOfBoundsErr> {
    if index >= VIEW_BUFFER_SIZE {
        return Err(OutOfBoundsErr);
    }
    let e: u16 = unsafe { read_volatile(VGA_BUFFER_ADDR.add(index)) };
    Ok(e)
}

pub struct Entry {
    color: u8,
    character: u8,
}

impl Entry {
    pub fn new(character: u8) -> Self {
        Entry {
            color: Color::Default as u8,
            character,
        }
    }

    pub fn to_u16(&self) -> u16 {
        ((self.color as u16) << 8) | (self.character as u16)
    }
}

#[repr(u8)]
enum Color {
    Default = 0x07,
}