use core::panic::PanicInfo;

use crate::{serial_print, serial_println};

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn() -> Result<(), &'static str>,
{
    fn run(&self) {
        serial_print!("{}...\t", ::core::any::type_name::<T>());
        match self() {
            Ok(()) => {
                serial_println!("[ok]");
            }
            Err(e) => {
                serial_println!("[failed]\n");
                serial_println!("Error: {}", e);
            }
        }
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    use crate::qemu;

    serial_println!("Running {} test(s)", tests.len());
    for test in tests {
        test.run();
    }
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}

// This panic handler is not marked as `#[panic_handler]` to make it
// importable by integration tests, who need to define their own
// panic handler.
// Marking this as `#[panic_handler]` would result in two panic handlers
// being compiled into the executable, which would of course fail.
pub fn panic_handler(info: &PanicInfo) -> ! {
    use crate::qemu;
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    unsafe { qemu::exit(qemu::ExitCode::Failed) };
    #[allow(clippy::empty_loop)]
    loop {}
}
