mod list;

#[cfg(test)]
mod test;

use std::rc::Rc;
use crate::list::*;

#[derive(Copy, Clone, Debug)]
pub struct AllocationInfo {
    pub base: u64,
    pub size: u64,
}

pub struct Allocator {
    pub info: AllocationInfo,
    list: Rc<List<AllocationInfo>>,
}

impl Allocator {
    pub fn new(base: u64, size: u64) -> Allocator {
        Allocator {
            info: AllocationInfo { base: base, size: size },
            list: List::cons(AllocationInfo { base: base, size: size }, &List::nil()),
        }
    }

    pub fn alloc(&mut self, size: u64) -> AllocationInfo {
        let old = &self.list;
        self.list = List::append(
            &List::cons(AllocationInfo { base: old.car().unwrap().base, size: size },
                        &List::cons(AllocationInfo { base: size, size: old.car().unwrap().size - size }, &List::nil())),
            old.cdr());
        self.list.car().unwrap()
    }
}

pub fn align_down(addr: u64, align: u64) -> u64 {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

pub fn align_up(addr: u64, align: u64) -> u64 {
    align_down(addr + align - 1, align)
}