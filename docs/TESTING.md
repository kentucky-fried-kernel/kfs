# Testing the kernel

Testing a kernel is not as easy as one could think, so let me walk you through the process of creating your own tests, and how they are being run.

This document should explain the basics of how this works, the actual implementation and the examples may differ a little, but I'm sure you can extrapolate.

## Table of Contents

- [How does it work?](#cant-i-just-use-cargo-test)
- [`x.py`](#x-py)
- [Unit Tests](#unit-tests)
- [E2E Tests](#end-to-end-tests)

## Can't I just use `cargo test`?

God I wish it were that simple. The main issue is that the `test` crate depends on the standard library, which we do not have access to here. Simply using `#[test]` in a `no_std` enviroment will give you the error: `can't find crate for 'test'`.

What one could do is build tests for a target with `std` and run them on a machine with an OS, but this would not allow testing for low-level kernel things like whether the GDT is set correctly, only high-level logic.

Luckily, creating our own `test` crate is not that hard.

## Creating our own `test` crate

The first thing we need to do is tell `rustc` to stop yelling at us, and let us cook. In order to communicate that, we will have our own test runner, we add the following attributes to our [lib.rs](/src/lib.rs) and/or [main.rs](/src/main.rs):

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

We still need to decide what to do when we panic. In order to communicate to QEMU that we want to exit, we need to run it with a device called `isa-debug-exit` by adding the following to our QEMU command line: `-device isa-debug-exit,iobase=0xf4,iosize=0x04`.

We can now [write an exit code to `0xf4`](/src/qemu.rs), which will make QEMU exit.

Putting everything together, we get something like this:

```rust
#![no_std]
#![no_main]
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

## `x.py`

The process described above is very manual, especially when you have a lot of different tests in the `./tests/` directory, where each one of them is its own binary. Furthermore, the hash in the generated artifacts is subject to change, so it can be annoying to fish it out of the target directory every time you run tests.

The [`x.py`](/x.py) script solves this by automating this whole process. It builds, discovers, and runs tests. This script comes with 2 mutually exclusive options: `--end-to-end-tests`, and `--unit-tests`, which build, discover, and run E2E tests, and unit tests, respectively.

If you need to run all tests, either run the script once with each of the options, or run `make test`.

## Unit Tests

By Unit Tests, I mean a test that runs in the same QEMU instance as all other Unit Tests. It is defined directly in a module, by giving the test function the `#[test_case]` attribute.

Example:

```rust
#[test_case]
fn it_works() -> Result<(), &'static str> {
    Ok(())
}
```

Note the difference to the standard `test` crate and the example above: the test returns a `Result<(), &'static str>` instead of just panicking on failure.

This is because with the above example, which was kept simple to focus on the important stuff, the first failing test would stop the whole test suite from running: It would cause a panic and stop the process early. In order to solve this, the [`test_runner`](/src/tester.rs#L29-L44) only accepts functions that conform to the [`Testable` trait](/src/tester.rs#L9-L27).

This allows the `test_runner` to run all tests and aggregate their results, no matter how many of them fail.

## End-to-end Tests

By E2E tests, I mean a suite of 1 or more tests that runs in its own QEMU instance, i.e., that does not interact with other test suites. It needs to define its own panic handler and kernel main in a Rust file in the `tests/` directory.

In a lot of cases, you do not want all your tests to run in the same system, since you might be doing weird stuff to trigger panics (or other shenanigans), which can mess with other, unrelated tests. This is where the end-to-end tests come in: Each one of them runs in its own QEMU instance.

To create an end-to-end test, create a new file in the [`./tests`](/tests) directory. This file should include its own panic handler and `kernel_main` functions, as well as all global attributes.

```rust
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use kfs::printkln;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kfs::tester::panic_handler(info)
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use kfs::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}
```

With this approach, you can use your custom `kernel_main` as a setup function, and define as many `#[test_case]`s as you want to make assertions about the state of the system. Note that all `#[test_case]`s defined in one end-to-end test suite run in the same binary/QEMU instance, they are only separated from other end-to-end test suites.

A cool use case can be checking for expected panics by defining a panic handler that exits with the success exit code, like so:

```rust
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kfs::tester::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kfs::qemu;
    unsafe { qemu::exit(qemu::ExitCode::Success) };
    loop {}
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    use kfs::qemu;
    test_main();
    unsafe { qemu::exit(qemu::ExitCode::Success) };
}

#[test_case]
fn should_panic() {
    assert!(false);
}
```
