pub mod gdt;
mod idt;

#[cfg(not(test))]
pub use idt::set_idt;
