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
use std::cell::RefCell;
use std::marker::{Sync, Send};

pub trait U8Access {
    fn write(&self, addr: u64, data: u8);
    fn read(&self, addr: u64) -> u8;
}

pub trait BytesAccess: U8Access {
    fn write(&self, addr: u64, data: &[u8]) {
        data.iter().enumerate().for_each(|(offset, d)| { U8Access::write(self, addr + offset as u64, *d) });
    }
    fn read(&self, addr: u64, data: &mut [u8]) {
        data.iter_mut().enumerate().for_each(|(offset, d)| { *d = U8Access::read(self, addr + offset as u64) });
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

pub trait IOAccess: U8Access + BytesAccess + U16Access + U32Access + U64Access + Sync + Send {}


struct Model {
    inner: RefCell<HashMap<u64, u8>>
}

impl Model {
    fn new() -> Model {
        Model { inner: RefCell::new(HashMap::new()) }
    }
}

impl U8Access for Model {
    fn write(&self, addr: u64, data: u8) {
        self.inner.borrow_mut().insert(addr, data);
    }

    fn read(&self, addr: u64) -> u8 {
        *self.inner.borrow_mut().entry(addr).
            or_insert_with(|| {
                let mut rng = rand::thread_rng();
                rng.gen()
            })
    }
}

impl BytesAccess for Model {}

impl U16Access for Model {}

impl U32Access for Model {}

impl U64Access for Model {}

enum Memory {
    Model(Arc<Mutex<Model>>),
    Block(Arc<Heap>),
    MMap(Arc<Region>),
    IO(Arc<Box<dyn IOAccess>>),
}

impl Memory {
    fn get_type(&self) -> String {
        match self {
            Memory::Model(_) => "Model".to_string(),
            Memory::Block(_) => "Block".to_string(),
            Memory::MMap(region) => format!("MMap({})", region.memory.get_type()),
            Memory::IO(_) => "IO".to_string(),
        }
    }
}

macro_rules! memory_access {
    ($x:ident, $f:ident, $obj:expr, $($p:expr),+) => {match $obj {
            Memory::IO(io) => $x::$f(io.deref().deref(),$($p,)+),
            Memory::Model(model) => $x::$f(model.lock().unwrap().deref(),$($p,)+),
            Memory::Block(heap) =>  $x::$f(heap.memory.deref(),$($p,)+),
            Memory::MMap(memory) => $x::$f(memory.deref(),$($p,)+),
        }
        }
}


impl U8Access for Memory {
    fn write(&self, addr: u64, data: u8) {
        memory_access!(U8Access, write, self, addr, data)
    }

    fn read(&self, addr: u64) -> u8 {
        memory_access!(U8Access, read, self, addr)
    }
}

impl U16Access for Memory {
    fn write(&self, addr: u64, data: u16) {
        memory_access!(U16Access, write, self, addr, data)
    }

    fn read(&self, addr: u64) -> u16 {
        memory_access!(U16Access, read, self, addr)
    }
}

impl U32Access for Memory {
    fn write(&self, addr: u64, data: u32) {
        memory_access!(U32Access, write, self, addr, data)
    }

    fn read(&self, addr: u64) -> u32 {
        memory_access!(U32Access, read, self, addr)
    }
}

impl U64Access for Memory {
    fn write(&self, addr: u64, data: u64) {
        memory_access!(U64Access, write,  self, addr, data)
    }

    fn read(&self, addr: u64) -> u64 {
        memory_access!(U64Access, read, self, addr)
    }
}

impl BytesAccess for Memory {
    fn write(&self, addr: u64, data: &[u8]) {
        memory_access!(BytesAccess, write, self, addr, data)
    }

    fn read(&self, addr: u64, data: &mut [u8]) {
        memory_access!(BytesAccess, read, self, addr, data)
    }
}

#[repr(C)]
pub struct Region {
    memory: Memory,
    pub info: MemInfo,
}

impl Region {
    pub fn get_type(&self) -> String {
        self.memory.get_type()
    }

    pub fn io(base: u64, size: u64, io: Box<dyn IOAccess>) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::IO(Arc::new(io)),
            info: MemInfo { base: base, size: size },
        })
    }

    fn model(base: u64, size: u64) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::Model(Arc::new(Mutex::new(Model::new()))),
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

    fn check_range(&self, addr: u64) {
        assert!(addr >= self.info.base && addr < self.info.base + self.info.size, format!("addr 0x{:x?} translate fail!range {:x?}", addr, self.info));
    }

    fn translate(&self, va: u64, size: usize) -> u64 {
        for addr in va..va + size as u64 {
            self.check_range(addr)
        }
        match &self.memory {
            Memory::MMap(memory) => va - self.info.base + memory.deref().info.base,
            _ => va,
        }
    }
}


impl U8Access for Region {
    fn write(&self, addr: u64, data: u8) {
        U8Access::write(&self.memory, self.translate(addr, 1), data)
    }

    fn read(&self, addr: u64) -> u8 {
        U8Access::read(&self.memory, self.translate(addr, 1))
    }
}

impl BytesAccess for Region {
    fn write(&self, addr: u64, data: &[u8]) {
        BytesAccess::write(&self.memory, self.translate(addr, data.len()), data)
    }

    fn read(&self, addr: u64, data: &mut [u8]) {
        BytesAccess::read(&self.memory, self.translate(addr, data.len()), data)
    }
}

impl U16Access for Region {
    fn write(&self, addr: u64, data: u16) {
        U16Access::write(&self.memory, self.translate(addr, 2), data)
    }

    fn read(&self, addr: u64) -> u16 {
        U16Access::read(&self.memory, self.translate(addr, 2))
    }
}

impl U32Access for Region {
    fn write(&self, addr: u64, data: u32) {
        U32Access::write(&self.memory, self.translate(addr, 4), data)
    }

    fn read(&self, addr: u64) -> u32 {
        U32Access::read(&self.memory, self.translate(addr, 4))
    }
}

impl U64Access for Region {
    fn write(&self, addr: u64, data: u64) {
        U64Access::write(&self.memory, self.translate(addr, 8), data)
    }

    fn read(&self, addr: u64) -> u64 {
        U64Access::read(&self.memory, self.translate(addr, 8))
    }
}

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
