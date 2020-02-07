mod list;

#[cfg(test)]
mod test;

use std::rc::Rc;
use crate::list::*;
use std::sync::Mutex;

#[cfg(test)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AllocationInfo {
    pub base: u64,
    pub size: u64,
}

#[cfg(not(test))]
#[derive(Copy, Clone)]
pub struct AllocationInfo {
    pub base: u64,
    pub size: u64,
}

pub struct Allocator {
    pub info: AllocationInfo,
    free_blocks: Mutex<Rc<List<AllocationInfo>>>,
}

impl Allocator {
    pub fn new(base: u64, size: u64) -> Allocator {
        Allocator {
            info: AllocationInfo { base: base, size: size },
            free_blocks: Mutex::new(List::cons(AllocationInfo { base: base, size: size }, &List::nil())),
        }
    }

    pub fn alloc(&self, size: u64, align: u64) -> Option<AllocationInfo> {
        let mut front = List::nil();

        let find_fn = |item: &&Rc<List<AllocationInfo>>| {
            let info = item.car().unwrap();
            let hit = info.size >= size + (align_up(info.base, align) - info.base);
            if !hit {
                front = List::cons(info, &front)
            } else {
                if align_up(info.base, align) != info.base {
                    front = List::cons(AllocationInfo { base: info.base, size: align_up(info.base, align) - info.base }, &front)
                }
                if info.size != size + (align_up(info.base, align) - info.base) {
                    front = List::cons(AllocationInfo { base: align_up(info.base, align) + size, size: info.size - size - (align_up(info.base, align) - info.base) }, &front)
                }
            }
            hit
        };

        {
            let mut list = self.free_blocks.lock().unwrap();
            let block = list.iter().find(find_fn);
            if let Some(item) = block {
                let result = Some(AllocationInfo { base: align_up(item.car().unwrap().base, align), size: size });
                *list = List::append(&front, item.cdr());
                result
            } else {
                None
            }
        }
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