extern crate terminus_spaceport_proc_macros;
#[macro_use]
extern crate lazy_static;

pub mod memory;

pub mod space;

pub mod irq;

pub mod virtio;

mod capi;

mod utils;
pub use utils::EXIT_CTRL;

pub mod devices;

#[cfg(test)]
mod test;
