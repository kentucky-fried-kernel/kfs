#![allow(clippy::manual_div_ceil)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
pub mod allocators;
mod init;
mod page;
mod page_allocator;
mod state;

pub use init::init;
pub use page::PAGE_SIZE;
