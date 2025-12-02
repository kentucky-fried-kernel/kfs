use core::panic::PanicInfo;

use crate::{serial_print, serial_println};

pub trait Testable {
    fn run(&self) -> bool;
}

impl<T> Testable for T
where
    T: Fn() -> Result<(), &'static str>,
{
    fn run(&self) -> bool {
        serial_print!("{}...\t", ::core::any::type_name::<T>());
        match self() {
            Ok(()) => {
                serial_println!("[ok]");
                true
            }
            Err(e) => {
                serial_println!("[failed]\n");
                serial_println!("Error: {}", e);
                false
            }
        }
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    use crate::qemu;

    serial_println!("Running {} test(s)", tests.len());
    let mut ok: usize = 0;
    let mut ko: usize = 0;
    for test in tests {
        if test.run() {
            ok += 1;
        } else {
            ko += 1;
        }
    }
    serial_println!("test result: {}. {} passed; {} failed.", if ko == 0 { "ok" } else { "FAILED" }, ok, ko);

    unsafe { qemu::exit(if ko == 0 { qemu::ExitCode::Success } else { qemu::ExitCode::Failed }) };
}

// This panic handler is not marked as `#[panic_handler]` to make it
// importable by integration tests, who need to define their own.
// Marking this as `#[panic_handler]` would result in a compilation
// failure.
pub fn panic_handler(info: &PanicInfo) -> ! {
    use crate::qemu;
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    unsafe { qemu::exit(qemu::ExitCode::Failed) };
    #[allow(clippy::empty_loop)]
    loop {}
}

// This panic handler function can be used as a `#[panic_handler]` for tests
// that are expected to panic.
pub fn should_panic_panic_handler() -> ! {
    use crate::qemu;
    serial_println!("[ok]\n");
    unsafe { qemu::exit(qemu::ExitCode::Success) };
    #[allow(clippy::empty_loop)]
    loop {}
}
