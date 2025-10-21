# Testing the kernel

Testing a kernel is not as easy as one could think, so let me walk you through the process of creating your own tests, and how they are being run.

This document should explain the basics of how this works, the actual implementation and the examples may differ a little, but I'm sure you can extrapolate.

## Table of Contents

- [How does it work?](#cant-i-just-use-cargo-test)
- [x.py](#x-py)
- [Unit Tests](#unit-tests)
- [E2E Tests](#end-to-end-tests)

## Can't I just use `cargo test`?

God I wish it were that simple. The main issue is that the `test` crate depends on the standard library, which we do not have access to here. Simply using `#[test]` in a `no_std` enviroment will give you the error: `can't find crate for 'test'`.

What one could do is build tests for a target with `std` and run them on a machine with an OS, but this would not allow testing for low-level kernel things like whether the GDT is set correctly, only high-level logic.

Luckily, creating our own `test` crate is not that hard.

## Creating our own `test` crate

The first thing we need to do is tell `rustc` to stop yelling at us and let us cook. In order to communicate that we will have our own test runner, we add the following attributes to our [lib.rs](/src/lib.rs) and/or [main.rs](/src/main.rs):

```rust
// /src/main.rs

// Allow us to create a custom test framework
#![feature(custom_test_frameworks)]
// This is the function you should use for testing
#![test_runner(crate::tester::test_runner)]
// Reexport the main function you generate as `test_main` so we can call it
#![reexport_test_harness_main = "test_main"]
```

The next step is to create a test main, the first function that is called after setting up the stack in [`_start`](/src/boot.rs#L36-L54). Our [non-test main](/src/main.rs#L9-L21) initializes everything and runs an interactive shell, which we do not want here: It should only run the tests and exit.

We can do this by conditionally compiling our `kernel_main` function based on the `cfg(test)` attribute:

```rust
// /src/main.rs
#[cfg(not(test))]
pub extern "C" fn kernel_main() {
    // Run normally
}

#[cfg(test)]
pub extern "C" fn kernel_main() {
    // Call the test_main function exported by rustc to run the tests.
    test_main();
}
```

Next, we want to be able to run those tests in the CI, so we should not open a QEMU window. This means we need two things:

1. [A println-like macro to write the test results to our console via serial ports](/src/serial.rs):

```rust
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
```

2. A command-line option to tell QEMU to run with no display, and redirect serial output to our host's stdio: `-serial stdio -display none`

We now need to create that `test_runner` function we told `rustc` to use. A test runner is very simple: it just needs to take in an array of test functions and run them.

```rust
// /src/tester.rs
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} test(s)", tests.len());
    for test in tests {
        test();
    }
}
```

We still need to decide what to do when we panic. In order to communicate to QEMU that we want to exit, we need to run it with a device called `isa-debug-exit` by adding the following to our QEMU command line `-device isa-debug-exit,iobase=0xf4,iosize=0x04`.

We can now [write an exit code to `0xf4`](/src/qemu.rs), which will make QEMU exit.

Putting everything together, we get something like this:

```rust
#![feature(custom_test_frameworks)]
#![test_runner(crate::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
pub extern "C" fn kernel_main() {
    use kfs::qemu;
    // Call the test_main function exported by rustc to run the tests.
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { qemu::exit(qemu::ExitCode::Failed) };
    loop {}
}
```

We can now add the following test to our main.rs:

```rust
#[test_case]
fn foo() {
    assert_eq!(1, 1);
}
```

Building the tests with `cargo build --tests --release` yields a test binary, which you have to grab from `./target/i386-unknown-none/release/deps/kfs-<16-chars-hash>`.

Use this binary as your `kernel.bin` in the [build process](/Makefile#L42-L46), and run it with this command:

```sh
qemu-system-i386 -cdrom build/kernel.iso -boot d -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -display none
```

You should get this output:

```
Running 1 test(s)
kfs::foo...	[ok]
```
