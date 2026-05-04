#![allow(clippy::manual_div_ceil)]
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::borrow_as_ptr)]
pub mod addressspace;
pub mod allocators;
pub mod init;
pub mod page;
pub mod page_allocator;
pub mod state;

pub use init::init;
pub use init::mmap_init;
pub use page::PAGE_SIZE;
