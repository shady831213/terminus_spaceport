#![crate_type = "dylib"]

mod list;
mod capi;
#[cfg(test)]
mod test;

use crate::list::*;
use std::sync::{Mutex, Arc};
use core::ops::Deref;

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

#[repr(C)]
pub struct Allocator {
    pub info: AllocationInfo,
    free_blocks: Arc<List<AllocationInfo>>,
    alloced_blocks: Arc<List<AllocationInfo>>,
}

impl Allocator {
    pub fn new(base: u64, size: u64) -> Allocator {
        Allocator {
            info: AllocationInfo { base: base, size: size },
            free_blocks: List::cons(AllocationInfo { base: base, size: size }, &List::nil()),
            alloced_blocks: List::nil(),
        }
    }

    pub fn alloc(&mut self, size: u64, align: u64) -> Option<AllocationInfo> {
        let mut front = List::nil();

        let find_fn = |item: &&Arc<List<AllocationInfo>>| {
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

        let block = self.free_blocks.iter().find(find_fn);
        if let Some(item) = block {
            let result = Some(AllocationInfo { base: align_up(item.car().unwrap().base, align), size: size });
            self.free_blocks = List::append(&front, item.cdr());
            self.alloced_blocks = List::cons(result.unwrap(), &self.alloced_blocks);
            result
        } else {
            None
        }
    }

    pub fn free(&mut self, addr: u64) {
        let mut front = List::nil();

        let find_fn = |item: &&Arc<List<AllocationInfo>>| {
            let info = item.car().unwrap();
            let hit = info.base == addr;
            if !hit {
                front = List::cons(info, &front)
            }
            hit
        };

        let block = self.alloced_blocks.iter().find(find_fn);
        if let Some(item) = block {
            let mut pre_front = List::nil();

            let pre_find_fn = |_item: &&Arc<List<AllocationInfo>>| {
                let info = _item.car().unwrap();
                let hit = info.base + info.size == item.car().unwrap().base;
                if !hit {
                    pre_front = List::cons(info, &pre_front)
                }
                hit
            };
            let pre_block = self.free_blocks.iter().find(pre_find_fn);
            let pre_info = if let Some(pre) = pre_block {
                let result = AllocationInfo { base: pre.car().unwrap().base, size: pre.car().unwrap().size + item.car().unwrap().size };
                self.free_blocks = List::append(&pre_front, pre.cdr());
                result
            } else {
                item.car().unwrap()
            };

            let mut post_front = List::nil();

            let post_find_fn = |_item: &&Arc<List<AllocationInfo>>| {
                let info = _item.car().unwrap();
                let hit = pre_info.base + pre_info.size == info.base;
                if !hit {
                    post_front = List::cons(info, &post_front)
                }
                hit
            };
            let post_block = self.free_blocks.iter().find(post_find_fn);
            let post_info = if let Some(post) = post_block {
                let result = AllocationInfo { base: pre_info.base, size: pre_info.size + post.car().unwrap().size };
                self.free_blocks = List::append(&post_front, post.cdr());
                result
            } else {
                pre_info
            };

            self.free_blocks = List::cons(post_info, &self.free_blocks);
            self.alloced_blocks = List::append(&front, item.cdr());
        } else {
            panic!(format!("invalid free @{}", addr));
        }
    }
}

pub struct LockedAllocator {
    inner: Mutex<Allocator>
}

impl LockedAllocator {
    pub fn new(base: u64, size: u64) -> LockedAllocator {
        LockedAllocator { inner: Mutex::new(Allocator::new(base, size)) }
    }
    pub fn alloc(&self, size: u64, align: u64) -> Option<AllocationInfo> {
        self.inner.lock().unwrap().alloc(size, align)
    }
}

impl Deref for LockedAllocator {
    type Target = Mutex<Allocator>;

    fn deref(&self) -> &Mutex<Allocator> {
        &self.inner
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