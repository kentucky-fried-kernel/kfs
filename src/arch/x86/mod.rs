mod gdt;
mod idt;

pub use gdt::set_gdt;
#[cfg(not(test))]
pub use idt::set_idt;
