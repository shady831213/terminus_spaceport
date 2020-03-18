extern crate terminus_spaceport_proc_macros;

pub use terminus_spaceport_proc_macros::*;

mod allocator;
mod model;

pub use model::*;

pub mod space;

mod capi;

#[cfg(test)]
mod test;

#[cfg(test)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MemInfo {
    pub base: u64,
    pub size: u64,
}

#[cfg(not(test))]
#[derive(Copy, Clone, Debug)]
pub struct MemInfo {
    pub base: u64,
    pub size: u64,
}

fn align_down(addr: u64, align: u64) -> u64 {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

fn align_up(addr: u64, align: u64) -> u64 {
    align_down(addr + align - 1, align)
}
