pub mod list;
#[cfg(test)]
mod test;

use list::*;
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
        let (block, free_blocks) = List::delete(&self.free_blocks, |item| {
            let info = item.car().unwrap();
            info.size >= size + (align_up(info.base, align) - info.base)
        });
        if let Some(item) = block {
            let info = item.car().unwrap();
            let result = Some(AllocationInfo { base: align_up(info.base, align), size: size });
            self.free_blocks = free_blocks;
            if align_up(info.base, align) != info.base {
                self.free_blocks = List::cons(AllocationInfo { base: info.base, size: align_up(info.base, align) - info.base }, &self.free_blocks)
            }
            if info.size != size + (align_up(info.base, align) - info.base) {
                self.free_blocks = List::cons(AllocationInfo { base: align_up(info.base, align) + size, size: info.size - size - (align_up(info.base, align) - info.base) }, &self.free_blocks)
            }
            self.alloced_blocks = List::cons(result.unwrap(), &self.alloced_blocks);
            result
        } else {
            None
        }
    }

    pub fn free(&mut self, addr: u64) {
        let (alloced_block, alloced_blocks) = List::delete(&self.alloced_blocks, |item| { item.car().unwrap().base == addr });
        if let Some(item) = alloced_block {
            let (pre_block, free_blocks) = List::delete(&self.free_blocks, |i| {
                let info = i.car().unwrap();
                info.base + info.size == item.car().unwrap().base
            });
            let pre_info = if let Some(pre) = pre_block {
                let info = pre.car().unwrap();
                self.free_blocks = free_blocks;
                AllocationInfo { base: info.base, size: info.size + item.car().unwrap().size }
            } else {
                item.car().unwrap()
            };

            let (post_block, free_blocks) = List::delete(&self.free_blocks, |i| {
                let info = i.car().unwrap();
                pre_info.base + pre_info.size == info.base
            });
            let post_info = if let Some(post) = post_block {
                let info = post.car().unwrap();
                self.free_blocks = free_blocks;
                AllocationInfo { base: pre_info.base, size: pre_info.size + info.size }
            } else {
                pre_info
            };

            self.free_blocks = List::cons(post_info, &self.free_blocks);
            self.alloced_blocks = alloced_blocks;
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
    pub fn free(&self, addr: u64) {
        self.inner.lock().unwrap().free(addr)
    }
}

impl Deref for LockedAllocator {
    type Target = Mutex<Allocator>;

    fn deref(&self) -> &Mutex<Allocator> {
        &self.inner
    }
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