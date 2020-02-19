use std::collections::HashMap;

mod allocator;
mod capi;

extern crate rand;

use rand::Rng;
use allocator::Allocator;
use std::sync::{Arc, Mutex};
use crate::allocator::AllocationInfo;
use std::borrow::BorrowMut;
use std::mem::size_of;
use std::convert::TryInto;
use std::ops::{Index, Deref, DerefMut};

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


pub trait U8MemAccess {
    fn write(&mut self, addr: u64, data: u8);
    fn read(&mut self, addr: u64) -> u8;
}

pub trait U16MemAccess {
    fn write(&mut self, addr: u64, data: u16);
    fn read(&mut self, addr: u64) -> u16;
}

pub trait U32MemAccess {
    fn write(&mut self, addr: u64, data: u32);
    fn read(&mut self, addr: u64) -> u32;
}

pub trait U64MemAccess {
    fn write(&mut self, addr: u64, data: u64);
    fn read(&mut self, addr: u64) -> u64;
}

trait BytesMemAccess: U8MemAccess {
    fn write(&mut self, base: u64, bytes: &[u8]) where {
        bytes.iter().enumerate().for_each(|(offset, data)| { U8MemAccess::write(self, base + offset as u64, *data) });
    }
    fn read(&mut self, base: u64, bytes: &mut [u8]) where {
        bytes.iter_mut().enumerate().for_each(|(offset, data)| { *data = U8MemAccess::read(self, base + offset as u64) });
    }
}

struct GlobalHeap {
    memroy: HashMap<u64, u8>,
    allocator: Allocator,
}

impl GlobalHeap {
    fn get() -> Arc<Mutex<GlobalHeap>> {
        static mut HEAP: Option<Arc<Mutex<GlobalHeap>>> = None;

        unsafe {
            HEAP.get_or_insert_with(|| {
                Arc::new(Mutex::new(GlobalHeap {
                    memroy: HashMap::new(),
                    allocator: Allocator::new(0, 0x8000000000000000),
                }))
            }).clone()
        }
    }
}

impl BytesMemAccess for GlobalHeap {}

impl U8MemAccess for GlobalHeap {
    fn write(&mut self, addr: u64, data: u8) {
        self.memroy.insert(addr, data);
    }

    fn read(&mut self, addr: u64) -> u8 {
        *self.memroy.
            entry(addr).
            or_insert_with(|| {
                let mut rng = rand::thread_rng();
                rng.gen()
            })
    }
}

impl U16MemAccess for GlobalHeap {
    fn write(&mut self, addr: u64, data: u16) {
        BytesMemAccess::write(self, align_down(addr, size_of::<u16>() as u64), &data.to_le_bytes());
    }

    fn read(&mut self, addr: u64) -> u16 {
        let base = align_down(addr, size_of::<u16>() as u64);
        let mut bytes = [0 as u8; size_of::<u16>()];
        BytesMemAccess::read(self, base, &mut bytes);
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl U32MemAccess for GlobalHeap {
    fn write(&mut self, addr: u64, data: u32) {
        BytesMemAccess::write(self, align_down(addr, size_of::<u32>() as u64), &data.to_le_bytes());
    }

    fn read(&mut self, addr: u64) -> u32 {
        let base = align_down(addr, size_of::<u32>() as u64);
        let mut bytes = [0 as u8; size_of::<u32>()];
        BytesMemAccess::read(self, base, &mut bytes);
        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl U64MemAccess for GlobalHeap {
    fn write(&mut self, addr: u64, data: u64) {
        BytesMemAccess::write(self, align_down(addr, size_of::<u64>() as u64), &data.to_le_bytes());
    }

    fn read(&mut self, addr: u64) -> u64 {
        let base = align_down(addr, size_of::<u64>() as u64);
        let mut bytes = [0 as u8; size_of::<u64>()];
        BytesMemAccess::read(self, base, &mut bytes);
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

struct MemRegion {
    allocator: Allocator,
    heap_map: AllocationInfo,
}

impl MemRegion {
    pub fn new(base: u64, size: u64) -> MemRegion {
        let info = GlobalHeap::get().lock().unwrap().allocator.alloc(size, 1);
        if let Some(map) = info {
            MemRegion {
                allocator: Allocator::new(base, size),
                heap_map: map,

            }
        } else {
            panic!(format!("can not add region {:?}", AllocationInfo { base: base, size: size }))
        }
    }

    fn va2pa(&self, va: u64) -> u64 {
        va - self.allocator.info.base + self.heap_map.base
    }
}

impl U8MemAccess for MemRegion {
    fn write(&mut self, addr: u64, data: u8) {
        U8MemAccess::write(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr), data)
    }

    fn read(&mut self, addr: u64) -> u8 {
        U8MemAccess::read(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr))
    }
}

impl U16MemAccess for MemRegion {
    fn write(&mut self, addr: u64, data: u16) {
        U16MemAccess::write(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr), data)
    }

    fn read(&mut self, addr: u64) -> u16 {
        U16MemAccess::read(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr))
    }
}

impl U32MemAccess for MemRegion {
    fn write(&mut self, addr: u64, data: u32) {
        U32MemAccess::write(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr), data)
    }

    fn read(&mut self, addr: u64) -> u32 {
        U32MemAccess::read(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr))
    }
}

impl U64MemAccess for MemRegion {
    fn write(&mut self, addr: u64, data: u64) {
        U64MemAccess::write(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr), data)
    }

    fn read(&mut self, addr: u64) -> u64 {
        U64MemAccess::read(GlobalHeap::get().lock().unwrap().deref_mut(),self.va2pa(addr))
    }
}