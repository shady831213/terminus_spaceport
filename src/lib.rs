extern crate terminus_spaceport_proc_macros;

pub use terminus_spaceport_proc_macros::*;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub mod memory;

pub mod space;

mod virtio;

mod capi;

#[cfg(test)]
mod test;

