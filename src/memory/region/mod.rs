#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::allocator::LockedAllocator;
use std::mem::size_of;
use std::convert::TryInto;
use std::ops::Deref;
use super::*;
use std::marker::{Sync, Send, Sized};
use std::hash::{BuildHasherDefault, Hasher};
use std::borrow::{BorrowMut, Borrow};

#[derive(Debug)]
pub enum Error {
    Misalign(u64),
    AccessErr(u64, String),
}

pub type Result<T> = std::result::Result<T, Error>;


pub trait U8Access {
    fn write(&self, addr: u64, data: u8) -> Result<()>;
    fn read(&self, addr: u64) -> Result<u8>;
}

pub trait BytesAccess: U8Access {
    fn write(&self, addr: u64, data: &[u8]) -> Result<()> {
        data.iter().enumerate().for_each(|(offset, d)| { U8Access::write(self, addr + offset as u64, *d) }.unwrap());
        Ok(())
    }
    fn read(&self, addr: u64, data: &mut [u8]) -> Result<()> {
        data.iter_mut().enumerate().for_each(|(offset, d)| { *d = U8Access::read(self, addr + offset as u64).unwrap() });
        Ok(())
    }
}


pub trait SizedAccess: BytesAccess {
    fn write<T: Sized>(&self, addr: u64, data: &T) -> Result<()> {
        BytesAccess::write(self, addr, unsafe { std::slice::from_raw_parts((data as *const T) as *const u8, std::mem::size_of::<T>()) })
    }

    fn read<T: Sized>(&self, addr: u64, data: &mut T) -> Result<()> {
        BytesAccess::read(self, addr, unsafe { std::slice::from_raw_parts_mut((data as *mut T) as *mut u8, std::mem::size_of::<T>()) })
    }
}

pub trait U16Access: BytesAccess {
    fn write(&self, addr: u64, data: u16) -> Result<()> {
        if addr.trailing_zeros() < 1 {
            return Err(Error::Misalign(addr));
        }
        BytesAccess::write(self, align_down(addr, size_of::<u16>() as u64), &data.to_le_bytes())
    }

    fn read(&self, addr: u64) -> Result<u16> {
        if addr.trailing_zeros() < 1 {
            return Err(Error::Misalign(addr));
        }
        let base = align_down(addr, size_of::<u16>() as u64);
        let mut bytes = [0 as u8; size_of::<u16>()];
        BytesAccess::read(self, base, &mut bytes)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }
}

pub trait U32Access: BytesAccess {
    fn write(&self, addr: u64, data: u32) -> Result<()> {
        if addr.trailing_zeros() < 2 {
            return Err(Error::Misalign(addr));
        }
        BytesAccess::write(self, align_down(addr, size_of::<u32>() as u64), &data.to_le_bytes())
    }

    fn read(&self, addr: u64) -> Result<u32> {
        if addr.trailing_zeros() < 2 {
            return Err(Error::Misalign(addr));
        }
        let base = align_down(addr, size_of::<u32>() as u64);
        let mut bytes = [0 as u8; size_of::<u32>()];
        BytesAccess::read(self, base, &mut bytes)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

pub trait U64Access: BytesAccess {
    fn write(&self, addr: u64, data: u64) -> Result<()> {
        if addr.trailing_zeros() < 3 {
            return Err(Error::Misalign(addr));
        }
        BytesAccess::write(self, align_down(addr, size_of::<u64>() as u64), &data.to_le_bytes())
    }

    fn read(&self, addr: u64) -> Result<u64> {
        if addr.trailing_zeros() < 3 {
            return Err(Error::Misalign(addr));
        }
        let base = align_down(addr, size_of::<u64>() as u64);
        let mut bytes = [0 as u8; size_of::<u64>()];
        BytesAccess::read(self, base, &mut bytes)?;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }
}

pub trait IOAccess: U8Access + BytesAccess + U16Access + U32Access + U64Access + Sync + Send {}

#[derive(Default)]
struct ModelHasher(u64);

impl Hasher for ModelHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        panic!("not implement")
    }
    fn write_u64(&mut self, i: u64) {
        self.0 = i
    }
}

struct LazyModel {
    inner: Mutex<HashMap<u64, u8, BuildHasherDefault<ModelHasher>>>
}

impl LazyModel {
    fn new() -> LazyModel {
        LazyModel { inner: Mutex::new(HashMap::default()) }
    }
}

impl U8Access for LazyModel {
    fn write(&self, addr: u64, data: u8) -> Result<()> {
        self.inner.lock().unwrap().insert(addr, data);
        Ok(())
    }

    fn read(&self, addr: u64) -> Result<u8> {
        Ok(if let Some(&v) = self.inner.lock().unwrap().get(&addr) {
            v
        } else {
            0
        })
    }
}

impl BytesAccess for LazyModel {
    fn write(&self, addr: u64, data: &[u8]) -> Result<()> {
        {
            let mut inner = self.inner.lock().unwrap();
            data.iter().enumerate().for_each(|(offset, d)| { inner.borrow_mut().insert(addr + offset as u64, *d); });
        }
        Ok(())
    }
    fn read(&self, addr: u64, data: &mut [u8]) -> Result<()> {
        {
            let inner = self.inner.lock().unwrap();
            data.iter_mut().enumerate().for_each(|(offset, d)| {
                *d = if let Some(&v) = inner.borrow().get(&(addr + offset as u64)) {
                    v
                } else {
                    0
                }
            });
        }
        Ok(())
    }
}

impl U16Access for LazyModel {}

impl U32Access for LazyModel {}

impl U64Access for LazyModel {}

struct Model {
    info: MemInfo,
    inner: Mutex<Box<[u8]>>,
}

impl Model {
    fn new(info: MemInfo) -> Model {
        let size = info.size;
        if std::mem::size_of::<u64>() != std::mem::size_of::<usize>() {
            assert!(size < 0x1_0000_0000, "global heap alloc max size can not exceed 4g when usize is 4, please use lazy_alloc!")
        }
        Model {
            info,
            inner: Mutex::new(vec![0; size as usize].into_boxed_slice()),
        }
    }
}

impl U8Access for Model {
    fn write(&self, addr: u64, data: u8) -> Result<()> {
        self.inner.lock().unwrap()[(addr - self.info.base) as usize] = data;
        Ok(())
    }

    fn read(&self, addr: u64) -> Result<u8> {
        Ok(self.inner.lock().unwrap()[(addr - self.info.base) as usize])
    }
}

impl BytesAccess for Model {
    fn write(&self, addr: u64, data: &[u8]) -> Result<()> {
        let offset = (addr - self.info.base) as usize;
        self.inner.lock().unwrap()[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    fn read(&self, addr: u64, data: &mut [u8]) -> Result<()> {
        let offset = (addr - self.info.base) as usize;
        data.copy_from_slice(&self.inner.lock().unwrap()[offset..offset + data.len()]);
        Ok(())
    }
}

impl U16Access for Model {}

impl U32Access for Model {}

impl U64Access for Model {}

struct Remap {
    region: Arc<Region>,
    info: MemInfo,
}

impl Remap {
    fn new(region: &Arc<Region>, info: MemInfo) -> Remap {
        Remap {
            region: Arc::clone(region),
            info,
        }
    }
}

enum Memory {
    Model(Model),
    LazyModel(LazyModel),
    Block(Arc<Heap>, Arc<Region>),
    RootBlock(Arc<Region>),
    Remap(Remap),
    IO(Arc<Box<dyn IOAccess>>),
}

impl Memory {
    fn get_type(&self) -> String {
        match self {
            Memory::Model(_) => "Model".to_string(),
            Memory::LazyModel(_) => "LazyModel".to_string(),
            Memory::Block(_, _) => "Block".to_string(),
            Memory::RootBlock(_) => "Block".to_string(),
            Memory::Remap(remap) => format!("Remap({}@{:#016x} -> {:#016x})", remap.region.memory.get_type(), remap.info.base, remap.info.base + remap.info.size),
            Memory::IO(_) => "IO".to_string(),
        }
    }
}

macro_rules! memory_access {
    ($x:ident, $f:ident, $obj:expr, $($p:expr),+) => {match $obj {
            Memory::IO(io) => $x::$f(io.deref().deref(),$($p,)+),
            Memory::Model(model) => $x::$f(model,$($p,)+),
            Memory::LazyModel(model) => $x::$f(model,$($p,)+),
            Memory::Block(_, region) =>  $x::$f(region.deref(),$($p,)+),
            Memory::RootBlock(region) =>  $x::$f(region.deref(),$($p,)+),
            Memory::Remap(remap) => $x::$f(remap.region.deref(),$($p,)+),
        }
        }
}


impl U8Access for Memory {
    fn write(&self, addr: u64, data: u8) -> Result<()> {
        memory_access!(U8Access, write, self, addr, data)
    }

    fn read(&self, addr: u64) -> Result<u8> {
        memory_access!(U8Access, read, self, addr)
    }
}

impl U16Access for Memory {
    fn write(&self, addr: u64, data: u16) -> Result<()> {
        memory_access!(U16Access, write, self, addr, data)
    }

    fn read(&self, addr: u64) -> Result<u16> {
        memory_access!(U16Access, read, self, addr)
    }
}

impl U32Access for Memory {
    fn write(&self, addr: u64, data: u32) -> Result<()> {
        memory_access!(U32Access, write, self, addr, data)
    }

    fn read(&self, addr: u64) -> Result<u32> {
        memory_access!(U32Access, read, self, addr)
    }
}

impl U64Access for Memory {
    fn write(&self, addr: u64, data: u64) -> Result<()> {
        memory_access!(U64Access, write,  self, addr, data)
    }

    fn read(&self, addr: u64) -> Result<u64> {
        memory_access!(U64Access, read, self, addr)
    }
}

impl BytesAccess for Memory {
    fn write(&self, addr: u64, data: &[u8]) -> Result<()> {
        memory_access!(BytesAccess, write, self, addr, data)
    }

    fn read(&self, addr: u64, data: &mut [u8]) -> Result<()> {
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

    fn lazy_model(base: u64, size: u64) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::LazyModel(LazyModel::new()),
            info: MemInfo { base: base, size: size },
        })
    }

    fn model(base: u64, size: u64) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::Model(Model::new(MemInfo { base: base, size: size })),
            info: MemInfo { base: base, size: size },
        })
    }

    fn block(base: u64, size: u64, heap: &Arc<Heap>, memory: &Arc<Region>) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::Block(Arc::clone(heap), Arc::clone(memory)),
            info: MemInfo { base: base, size: size },
        })
    }

    fn root_block(base: u64, size: u64, memory: &Arc<Region>) -> Arc<Region> {
        Arc::new(Region {
            memory: Memory::RootBlock(Arc::clone(memory)),
            info: MemInfo { base: base, size: size },
        })
    }

    pub fn remap(base: u64, memory: &Arc<Region>) -> Arc<Region> {
        let info = memory.info;
        Arc::new(Region {
            memory: Memory::Remap(Remap::new(memory, info)),
            info: MemInfo { base: base, size: info.size },
        })
    }

    pub fn remap_partial(base: u64, memory: &Arc<Region>, offset: u64, size: u64) -> Arc<Region> {
        assert!(offset + size <= memory.info.size);
        let info = memory.info;
        Arc::new(Region {
            memory: Memory::Remap(Remap::new(memory, MemInfo { base: info.base + offset, size: size })),
            info: MemInfo { base: base, size: size },
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
            Memory::Remap(remap) => va - self.info.base + remap.info.base,
            _ => va,
        }
    }
}


impl U8Access for Region {
    fn write(&self, addr: u64, data: u8) -> Result<()> {
        U8Access::write(&self.memory, self.translate(addr, 1), data)
    }

    fn read(&self, addr: u64) -> Result<u8> {
        U8Access::read(&self.memory, self.translate(addr, 1))
    }
}

impl BytesAccess for Region {
    fn write(&self, addr: u64, data: &[u8]) -> Result<()> {
        BytesAccess::write(&self.memory, self.translate(addr, data.len()), data)
    }

    fn read(&self, addr: u64, data: &mut [u8]) -> Result<()> {
        BytesAccess::read(&self.memory, self.translate(addr, data.len()), data)
    }
}

impl U16Access for Region {
    fn write(&self, addr: u64, data: u16) -> Result<()>  {
        if addr.trailing_zeros() < 1 {
            return Err(Error::Misalign(addr));
        }
        let pa = self.translate(addr, 2);
        if pa & 0x1 == 0 {
            U16Access::write(&self.memory, pa, data)
        } else {
            BytesAccess::write(&self.memory, pa, &data.to_le_bytes())
        }
    }

    fn read(&self, addr: u64) -> Result<u16>  {
        if addr.trailing_zeros() < 1 {
            return Err(Error::Misalign(addr));
        }
        let pa = self.translate(addr, 2);
        if pa & 0x1 == 0 {
            U16Access::read(&self.memory, pa)
        } else {
            let mut bytes = [0 as u8; size_of::<u16>()];
            BytesAccess::read(&self.memory, pa, &mut bytes)?;
            Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
        }
    }
}

impl U32Access for Region {
    fn write(&self, addr: u64, data: u32) -> Result<()>  {
        if addr.trailing_zeros() < 2 {
            return Err(Error::Misalign(addr));
        }
        let pa = self.translate(addr, 4);
        if pa & 0x3 == 0 {
            U32Access::write(&self.memory, pa, data)
        } else {
            BytesAccess::write(&self.memory, pa, &data.to_le_bytes())
        }
    }

    fn read(&self, addr: u64) -> Result<u32>  {
        if addr.trailing_zeros() < 2 {
            return Err(Error::Misalign(addr));
        }
        let pa = self.translate(addr, 4);
        if pa & 0x3 == 0 {
            U32Access::read(&self.memory, pa)
        } else {
            let mut bytes = [0 as u8; size_of::<u32>()];
            BytesAccess::read(&self.memory, pa, &mut bytes)?;
            Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
        }
    }
}

impl U64Access for Region {
    fn write(&self, addr: u64, data: u64) -> Result<()>  {
        if addr.trailing_zeros() < 3 {
            return Err(Error::Misalign(addr));
        }
        let pa = self.translate(addr, 8);
        if pa & 0x7 == 0 {
            U64Access::write(&self.memory, pa, data)
        } else {
            BytesAccess::write(&self.memory, pa, &data.to_le_bytes())
        }
    }

    fn read(&self, addr: u64) -> Result<u64> {
        if addr.trailing_zeros() < 3 {
            return Err(Error::Misalign(addr));
        }
        let pa = self.translate(addr, 8);
        if pa & 0x7 == 0 {
            U64Access::read(&self.memory, pa)
        } else {
            let mut bytes = [0 as u8; size_of::<u64>()];
            BytesAccess::read(&self.memory, pa, &mut bytes)?;
            Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
        }
    }
}

impl SizedAccess for Region {}

impl Drop for Region {
    fn drop(&mut self) {
        if let Memory::Block(heap, _) = &self.memory {
            heap.free(self.info.base)
        } else if let Memory::RootBlock(_) = &self.memory {
            GHEAP.free(self.info.base)
        }
    }
}

trait Free {
    fn free(&self, addr: u64);
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
    pub fn new(memory: &Arc<Region>) -> Arc<Heap> {
        Arc::new(Heap {
            memory: Arc::clone(memory),
            allocator: LockedAllocator::new(memory.info.base, memory.info.size),
        })
    }

    pub fn alloc(self: &Arc<Self>, size: u64, align: u64) -> std::result::Result<Arc<Region>, String> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Ok(Region::block(info.base, info.size, self, &self.memory))
        } else {
            Err("oom!".to_string())
        }
    }
    pub fn get_region(&self) -> &Arc<Region> {
        &self.memory
    }
}

impl Free for Heap {
    fn free(&self, addr: u64) {
        self.allocator.free(addr)
    }
}

lazy_static! {
    pub static ref GHEAP:GlobalHeap =GlobalHeap{allocator: LockedAllocator::new(0, 0x8000_0000_0000_0000)};
}

#[cfg(test)]
pub struct GlobalHeap {
    pub allocator: LockedAllocator,
}

#[cfg(not(test))]
pub struct GlobalHeap {
    allocator: LockedAllocator,
}

impl GlobalHeap {
    pub fn lazy_alloc(&self, size: u64, align: u64) -> std::result::Result<Arc<Region>, String> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Ok(Region::root_block(info.base, info.size, &Region::lazy_model(info.base, info.size)))
        } else {
            Err("oom!".to_string())
        }
    }

    pub fn alloc(&self, size: u64, align: u64) -> std::result::Result<Arc<Region>, String> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Ok(Region::root_block(info.base, info.size, &Region::model(info.base, info.size)))
        } else {
            Err("oom!".to_string())
        }
    }
}

impl Free for GlobalHeap {
    fn free(&self, addr: u64) {
        self.allocator.free(addr)
    }
}