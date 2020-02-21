use std::collections::HashMap;

mod allocator;
mod model;
mod capi;
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
