pub mod list;
#[cfg(test)]
mod test;

use list::*;
use super::*;
use std::sync::{Mutex, Arc};
use core::ops::Deref;

#[cfg(test)]
#[repr(C)]
pub struct Allocator {
    pub info: MemInfo,
    pub free_blocks: Arc<List<MemInfo>>,
    pub alloced_blocks: Arc<List<MemInfo>>,
}

#[cfg(not(test))]
#[repr(C)]
pub struct Allocator {
    pub info: MemInfo,
    free_blocks: Arc<List<MemInfo>>,
    alloced_blocks: Arc<List<MemInfo>>,
}

impl Allocator {
    pub fn new(base: u64, size: u64) -> Allocator {
        Allocator {
            info: MemInfo { base: base, size: size },
            free_blocks: List::cons(MemInfo { base: base, size: size }, &List::nil()),
            alloced_blocks: List::nil(),
        }
    }

    pub fn alloc(&mut self, size: u64, align: u64) -> Option<MemInfo> {
        let (block, free_blocks) = List::delete(&self.free_blocks, |item| {
            let info = item.car().unwrap();
            info.size >= size + (align_up(info.base, align) - info.base)
        });
        if let Some(item) = block {
            let info = item.car().unwrap();
            let result = Some(MemInfo { base: align_up(info.base, align), size: size });
            self.free_blocks = free_blocks;
            if align_up(info.base, align) != info.base {
                self.free_blocks = List::cons(MemInfo { base: info.base, size: align_up(info.base, align) - info.base }, &self.free_blocks)
            }
            if info.size != size + (align_up(info.base, align) - info.base) {
                self.free_blocks = List::cons(MemInfo { base: align_up(info.base, align) + size, size: info.size - size - (align_up(info.base, align) - info.base) }, &self.free_blocks)
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
                MemInfo { base: info.base, size: info.size + item.car().unwrap().size }
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
                MemInfo { base: pre_info.base, size: pre_info.size + info.size }
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
    pub fn alloc(&self, size: u64, align: u64) -> Option<MemInfo> {
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

