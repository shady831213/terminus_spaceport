use std::collections::HashMap;

mod allocator;
mod capi;

extern crate rand;

use rand::Rng;
use allocator::Allocator;
use std::sync::{Arc, Mutex};
use crate::allocator::{AllocationInfo, LockedAllocator};
use std::borrow::{BorrowMut, Borrow};
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
    fn write(&self, addr: u64, data: u8);
    fn read(&self, addr: u64) -> u8;
}

pub trait BytesMemAccess: U8MemAccess {
    fn write(&self, base: u64, bytes: &[u8]) where {
        bytes.iter().enumerate().for_each(|(offset, data)| { U8MemAccess::write(self, base + offset as u64, *data) });
    }
    fn read(&self, base: u64, bytes: &mut [u8]) where {
        bytes.iter_mut().enumerate().for_each(|(offset, data)| { *data = U8MemAccess::read(self, base + offset as u64) });
    }
}


pub trait U16MemAccess: BytesMemAccess {
    fn write(&self, addr: u64, data: u16) {
        BytesMemAccess::write(self, align_down(addr, size_of::<u16>() as u64), &data.to_le_bytes());
    }

    fn read(&self, addr: u64) -> u16 {
        let base = align_down(addr, size_of::<u16>() as u64);
        let mut bytes = [0 as u8; size_of::<u16>()];
        BytesMemAccess::read(self, base, &mut bytes);
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
}

pub trait U32MemAccess: BytesMemAccess {
    fn write(&self, addr: u64, data: u32) {
        BytesMemAccess::write(self, align_down(addr, size_of::<u32>() as u64), &data.to_le_bytes());
    }

    fn read(&self, addr: u64) -> u32 {
        let base = align_down(addr, size_of::<u32>() as u64);
        let mut bytes = [0 as u8; size_of::<u32>()];
        BytesMemAccess::read(self, base, &mut bytes);
        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

pub trait U64MemAccess: BytesMemAccess {
    fn write(&self, addr: u64, data: u64) {
        BytesMemAccess::write(self, align_down(addr, size_of::<u64>() as u64), &data.to_le_bytes());
    }

    fn read(&self, addr: u64) -> u64 {
        let base = align_down(addr, size_of::<u64>() as u64);
        let mut bytes = [0 as u8; size_of::<u64>()];
        BytesMemAccess::read(self, base, &mut bytes);
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

enum Memory {
    Model(Arc<Mutex<HashMap<u64, u8>>>),
    Block(Arc<Heap>),
    MMap(Arc<Region>),
}

impl U8MemAccess for Memory {
    fn write(&self, addr: u64, data: u8) {
        match self {
            Memory::Model(model) => { model.lock().unwrap().insert(addr, data); }
            Memory::Block(heap) => U8MemAccess::write(heap.memory.deref(), addr, data),
            Memory::MMap(memory) => U8MemAccess::write(memory.deref(), addr, data),
        }
    }

    fn read(&self, addr: u64) -> u8 {
        match self {
            Memory::Model(model) => {
                *model.lock().unwrap().
                    entry(addr).
                    or_insert_with(|| {
                        let mut rng = rand::thread_rng();
                        rng.gen()
                    })
            }
            Memory::Block(heap) => U8MemAccess::read(heap.memory.deref(), addr),
            Memory::MMap(memory) => U8MemAccess::read(memory.deref(), addr),
        }
    }
}

struct Region {
    memory: Memory,
    info: AllocationInfo,
}

impl Region {
    fn model(base: u64, size: u64) -> Region {
        Region {
            memory: Memory::Model(Arc::new(Mutex::new(HashMap::new()))),
            info: AllocationInfo { base: base, size: size },
        }
    }

    fn block(base: u64, size: u64, memory: Arc<Heap>) -> Region {
        Region {
            memory: Memory::Block(Arc::clone(&memory)),
            info: AllocationInfo { base: base, size: size },
        }
    }

    fn mmap(base: u64, memory: Arc<Region>) -> Region {
        let info = memory.info;
        Region {
            memory: Memory::MMap(Arc::clone(&memory)),
            info: AllocationInfo { base: base, size: info.size },
        }
    }

    fn translate(&self, va: u64) -> u64 {
        assert!(va >= self.info.base && va < self.info.base + self.info.size, format!("addr {} translate fail!range {:?}", va, self.info));
        match &self.memory {
            Memory::Block(_) | Memory::Model(_) => va,
            Memory::MMap(memory) => va - self.info.base + memory.deref().info.base
        }
    }
}

impl BytesMemAccess for Region {}

impl U8MemAccess for Region {
    fn write(&self, addr: u64, data: u8) {
        self.memory.write(self.translate(addr), data)
    }

    fn read(&self, addr: u64) -> u8 {
        self.memory.read(self.translate(addr))
    }
}

impl U16MemAccess for Region {}

impl U32MemAccess for Region {}

impl U64MemAccess for Region {}
//fixme??
impl Drop for Region {
    fn drop(&mut self) {
        if let Memory::Block(heap) = &self.memory {
            heap.allocator.free(self.info.base)
        }
    }
}


struct Heap {
    memory: Arc<Region>,
    allocator: LockedAllocator,
}

impl Heap {
    fn global() -> Arc<Heap> {
        static mut HEAP: Option<Arc<Heap>> = None;

        unsafe {
            HEAP.get_or_insert_with(|| {
                Arc::new(Self::new(Arc::new(Region::model(0, 0x8000000000000000))))
            }).clone()
        }
    }
    fn new(memory: Arc<Region>) -> Heap {
        Heap {
            memory: Arc::clone(&memory),
            allocator: LockedAllocator::new(memory.info.base, memory.info.size),
        }
    }

    fn alloc(self: Arc<Self>, size: u64, align: u64) -> Arc<Region> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Arc::new(Region::block(info.base, info.size, Arc::clone(&self)))
        } else {
            panic!("oom!")
        }
    }
}

