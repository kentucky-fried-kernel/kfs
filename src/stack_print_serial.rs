use crate::{
    boot::{STACK, STACK_SIZE},
    serial_print,
};

fn get_stack_pointer() -> u32 {
    let sp: usize;
    unsafe {
        core::arch::asm!(
            "mov {0}, esp",
            out(reg) sp,
        );
    }

    sp as u32
}

#[allow(static_mut_refs)]
pub fn print_stack_to_serial() {
    let sp_addr = get_stack_pointer();
    let st = unsafe { (STACK.as_ptr() as usize + STACK_SIZE) as *const u8 as u32 };
    let mut row: [u8; 16];

    assert!(sp_addr <= st);

    for row_idx in (sp_addr..st).step_by(16) {
        let ptr = row_idx as *const u8;
        row = unsafe { *(ptr.cast::<[u8; 16]>()) };
        dump_row(row, ptr);
    }
}

fn dump_row(row: [u8; 16], ptr: *const u8) {
    serial_print!("{:08x}-{:08x} ", ptr as u32, ptr as u32 + 15);
    for word in row.chunks(4) {
        // Reminder to future self: casting to u32 prints the bytes in little-endian.
        for byte in word {
            serial_print!("{:02x}", byte);
        }
        serial_print!(" ");
    }
    for byte in row {
        serial_print!("{}", (if (32..127).contains(&byte) { byte } else { b'.' }) as char);
    }
    serial_print!("\n");
}
