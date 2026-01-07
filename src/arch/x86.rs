#![deny(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::wildcard_enum_match_arm)]

pub mod exception;
pub mod gdt;
pub mod idt;
pub mod irq;
pub mod pic;
