#[cfg(test)]
mod test;

extern crate rand;
use std::collections::HashMap;
use rand::Rng;
use std::sync::{Arc, Mutex};
use crate::allocator::LockedAllocator;
use std::mem::size_of;
use std::convert::TryInto;
use std::ops::Deref;
use super::*;

pub trait U8Access {
    fn write(&self, addr: u64, data: u8);
    fn read(&self, addr: u64) -> u8;
}

pub trait BytesAccess: U8Access {
    fn write(&self, base: u64, bytes: &[u8]) where {
        bytes.iter().enumerate().for_each(|(offset, data)| { U8Access::write(self, base + offset as u64, *data) });
    }
    fn read(&self, base: u64, bytes: &mut [u8]) where {
        bytes.iter_mut().enumerate().for_each(|(offset, data)| { *data = U8Access::read(self, base + offset as u64) });
    }
}


pub trait U16Access: BytesAccess {
    fn write(&self, addr: u64, data: u16) {
        BytesAccess::write(self, align_down(addr, size_of::<u16>() as u64), &data.to_le_bytes());
    }

    fn read(&self, addr: u64) -> u16 {
        let base = align_down(addr, size_of::<u16>() as u64);
        let mut bytes = [0 as u8; size_of::<u16>()];
        BytesAccess::read(self, base, &mut bytes);
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
}

pub trait U32Access: BytesAccess {
    fn write(&self, addr: u64, data: u32) {
        BytesAccess::write(self, align_down(addr, size_of::<u32>() as u64), &data.to_le_bytes());
    }

    fn read(&self, addr: u64) -> u32 {
        let base = align_down(addr, size_of::<u32>() as u64);
        let mut bytes = [0 as u8; size_of::<u32>()];
        BytesAccess::read(self, base, &mut bytes);
        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

pub trait U64Access: BytesAccess {
    fn write(&self, addr: u64, data: u64) {
        BytesAccess::write(self, align_down(addr, size_of::<u64>() as u64), &data.to_le_bytes());
    }

    fn read(&self, addr: u64) -> u64 {
        let base = align_down(addr, size_of::<u64>() as u64);
        let mut bytes = [0 as u8; size_of::<u64>()];
        BytesAccess::read(self, base, &mut bytes);
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

enum Memory {
    Model(Arc<Mutex<HashMap<u64, u8>>>),
    Block(Arc<Heap>),
    MMap(Arc<Region>),
}

impl U8Access for Memory {
    fn write(&self, addr: u64, data: u8) {
        match self {
            Memory::Model(model) => { model.lock().unwrap().insert(addr, data); }
            Memory::Block(heap) => U8Access::write(heap.memory.deref(), addr, data),
            Memory::MMap(memory) => U8Access::write(memory.deref(), addr, data),
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
            Memory::Block(heap) => U8Access::read(heap.memory.deref(), addr),
            Memory::MMap(memory) => U8Access::read(memory.deref(), addr),
        }
    }
}

#[repr(C)]
pub struct Region {
    memory: Memory,
    pub info: MemInfo,
}

impl Region {
    fn model(base: u64, size: u64) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::Model(Arc::new(Mutex::new(HashMap::new()))),
            info: MemInfo { base: base, size: size },
        })
    }

    fn block(base: u64, size: u64, memory: &Arc<Heap>) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::Block(Arc::clone(memory)),
            info: MemInfo { base: base, size: size },
        })
    }

    pub fn mmap(base: u64, memory: &Arc<Region>) -> Arc<Region> {
        let info = memory.info;
        Arc::new(Region {
            memory: Memory::MMap(Arc::clone(memory)),
            info: MemInfo { base: base, size: info.size },
        })
    }

    fn translate(&self, va: u64) -> u64 {
        assert!(va >= self.info.base && va < self.info.base + self.info.size, format!("addr 0x{:x?} translate fail!range {:?}", va, self.info));
        match &self.memory {
            Memory::Block(_) | Memory::Model(_) => va,
            Memory::MMap(memory) => va - self.info.base + memory.deref().info.base
        }
    }
}

impl BytesAccess for Region {}

impl U8Access for Region {
    fn write(&self, addr: u64, data: u8) {
        self.memory.write(self.translate(addr), data)
    }

    fn read(&self, addr: u64) -> u8 {
        self.memory.read(self.translate(addr))
    }
}

impl U16Access for Region {}

impl U32Access for Region {}

impl U64Access for Region {}

impl Drop for Region {
    fn drop(&mut self) {
        if let Memory::Block(heap) = &self.memory {
            heap.allocator.free(self.info.base)
        }
    }
}

#[cfg(test)]
#[repr(C)]
pub struct Heap {
    memory: Arc<Region>,
    pub allocator: LockedAllocator,
}

#[cfg(not(test))]
#[repr(C)]
pub struct Heap {
    memory: Arc<Region>,
    allocator: LockedAllocator,
}

impl Heap {
    pub fn global() -> Arc<Heap> {
        static mut HEAP: Option<Arc<Heap>> = None;

        unsafe {
            HEAP.get_or_insert_with(|| {
                Self::new(&Region::model(0, 0x8000000000000000))
            }).clone()
        }
    }
    pub fn new(memory: &Arc<Region>) -> Arc<Heap> {
        Arc::new(Heap {
            memory: Arc::clone(memory),
            allocator: LockedAllocator::new(memory.info.base, memory.info.size),
        })
    }

    pub fn alloc(self: &Arc<Self>, size: u64, align: u64) -> Arc<Region> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Region::block(info.base, info.size, self)
        } else {
            panic!("oom!")
        }
    }
}
