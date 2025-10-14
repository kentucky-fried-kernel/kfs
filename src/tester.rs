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
