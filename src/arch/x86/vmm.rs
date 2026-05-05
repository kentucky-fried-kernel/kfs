#![allow(clippy::manual_div_ceil)]
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::borrow_as_ptr)]
pub mod addressspace;
mod allocators;
mod init;
mod page;
mod page_allocator;
mod state;

pub use init::init;
pub use page::PAGE_SIZE;
