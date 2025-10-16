#![allow(unused_imports)]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use kfs::terminal::{
        Screen,
        vga::{Buffer, Color},
    };

    let mut s = Screen::default();
    s.write_str("KERNEL PANIC\n");
    s.write_str(_info.message().as_str().unwrap_or("???"));
    let b = Buffer::from_screen(&s);
    b.flush();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kfs::{qemu, serial_println};

    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    unsafe { qemu::exit(qemu::ExitCode::Failed) };
    loop {}
}
