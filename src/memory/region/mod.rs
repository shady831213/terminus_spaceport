#[cfg(test)]
mod test;

use super::*;
use crate::memory::allocator::{Allocator, LockedAllocator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::Sized;
use std::mem::size_of;
use std::ops::Deref;
use std::rc::Rc;

pub trait BytesAccess {
    fn write(&self, addr: &u64, data: &[u8]) -> Result<usize, String>;
    fn read(&self, addr: &u64, data: &mut [u8]) -> Result<usize, String>;
}

pub trait U8Access {
    fn write(&self, addr: &u64, data: u8);
    fn read(&self, addr: &u64) -> u8;
}

pub trait SizedAccess: BytesAccess {
    fn write<T: Sized>(&self, addr: &u64, data: &T) {
        BytesAccess::write(self, addr, unsafe {
            std::slice::from_raw_parts((data as *const T) as *const u8, std::mem::size_of::<T>())
        })
        .unwrap();
    }

    fn read<T: Sized>(&self, addr: &u64, data: &mut T) {
        BytesAccess::read(self, addr, unsafe {
            std::slice::from_raw_parts_mut((data as *mut T) as *mut u8, std::mem::size_of::<T>())
        })
        .unwrap();
    }
}

pub trait U16Access: BytesAccess {
    fn write(&self, addr: &u64, data: u16) {
        assert!(
            addr.trailing_zeros() > 0,
            format!("U16Access:unaligned addr:{:#x}", *addr)
        );
        BytesAccess::write(self, addr, &data.to_le_bytes()).unwrap();
    }

    fn read(&self, addr: &u64) -> u16 {
        assert!(
            addr.trailing_zeros() > 0,
            format!("U16Access:unaligned addr:{:#x}", *addr)
        );
        let mut bytes = [0 as u8; size_of::<u16>()];
        BytesAccess::read(self, addr, &mut bytes).unwrap();
        u16::from_le_bytes(bytes)
    }
}

pub trait U32Access: BytesAccess {
    fn write(&self, addr: &u64, data: u32) {
        assert!(
            addr.trailing_zeros() > 1,
            format!("U32Access:unaligned addr:{:#x}", *addr)
        );
        BytesAccess::write(self, addr, &data.to_le_bytes()).unwrap();
    }

    fn read(&self, addr: &u64) -> u32 {
        assert!(
            addr.trailing_zeros() > 1,
            format!("U32Access:unaligned addr:{:#x}", *addr)
        );
        let mut bytes = [0 as u8; size_of::<u32>()];
        BytesAccess::read(self, addr, &mut bytes).unwrap();
        u32::from_le_bytes(bytes)
    }
}

pub trait U64Access: BytesAccess {
    fn write(&self, addr: &u64, data: u64) {
        assert!(
            addr.trailing_zeros() > 2,
            format!("U64Access:unaligned addr:{:#x}", *addr)
        );
        BytesAccess::write(self, addr, &data.to_le_bytes()).unwrap();
    }

    fn read(&self, addr: &u64) -> u64 {
        assert!(
            addr.trailing_zeros() > 2,
            format!("U64Access:unaligned addr:{:#x}", *addr)
        );
        let mut bytes = [0 as u8; size_of::<u64>()];
        BytesAccess::read(self, addr, &mut bytes).unwrap();
        u64::from_le_bytes(bytes)
    }
}

pub trait IOAccess: U8Access + BytesAccess + U16Access + U32Access + U64Access {}

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
    inner: RefCell<HashMap<u64, u8, BuildHasherDefault<ModelHasher>>>,
}

impl LazyModel {
    fn new() -> LazyModel {
        LazyModel {
            inner: RefCell::new(HashMap::default()),
        }
    }
}

impl U8Access for LazyModel {
    fn write(&self, addr: &u64, data: u8) {
        self.inner.borrow_mut().insert(*addr, data);
    }

    fn read(&self, addr: &u64) -> u8 {
        if let Some(&v) = self.inner.borrow().get(addr) {
            v
        } else {
            0
        }
    }
}

impl BytesAccess for LazyModel {
    fn write(&self, addr: &u64, data: &[u8]) -> Result<usize, String> {
        {
            data.iter().enumerate().for_each(|(offset, d)| {
                self.inner.borrow_mut().insert(*addr + offset as u64, *d);
            });
        }
        Ok(data.len())
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> Result<usize, String> {
        {
            data.iter_mut().enumerate().for_each(|(offset, d)| {
                *d = if let Some(&v) = self.inner.borrow().get(&(*addr + offset as u64)) {
                    v
                } else {
                    0
                }
            });
        }
        Ok(data.len())
    }
}

impl U16Access for LazyModel {}

impl U32Access for LazyModel {}

impl U64Access for LazyModel {}

struct Model {
    info: MemInfo,
    inner: RefCell<Box<[u8]>>,
}

impl Model {
    fn new(info: MemInfo) -> Model {
        let size = info.size;
        if std::mem::size_of::<u64>() != std::mem::size_of::<usize>() {
            assert!(size < 0x1_0000_0000, "global heap alloc max size can not exceed 4g when usize is 4, please use lazy_alloc!")
        }
        Model {
            info,
            inner: RefCell::new(vec![0; size as usize].into_boxed_slice()),
        }
    }
}

impl U8Access for Model {
    fn write(&self, addr: &u64, data: u8) {
        self.inner.borrow_mut()[(*addr - self.info.base) as usize] = data;
    }

    fn read(&self, addr: &u64) -> u8 {
        self.inner.borrow()[(*addr - self.info.base) as usize]
    }
}

impl BytesAccess for Model {
    fn write(&self, addr: &u64, data: &[u8]) -> Result<usize, String> {
        let offset = (*addr - self.info.base) as usize;
        self.inner.borrow_mut()[offset..offset + data.len()].copy_from_slice(data);
        Ok(data.len())
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> Result<usize, String> {
        let offset = (*addr - self.info.base) as usize;
        data.copy_from_slice(&self.inner.borrow()[offset..offset + data.len()]);
        Ok(data.len())
    }
}

impl U16Access for Model {}

impl U32Access for Model {}

impl U64Access for Model {}

struct Remap {
    region: Rc<Region>,
    info: MemInfo,
}

impl Remap {
    fn new(region: &Rc<Region>, info: MemInfo) -> Remap {
        Remap {
            region: Rc::clone(region),
            info,
        }
    }
}

enum Memory {
    Model(Model),
    LazyModel(LazyModel),
    Block(Rc<Heap>, Rc<Region>),
    RootBlock(Box<Region>),
    Remap(Remap),
    IO(Box<dyn IOAccess>),
}

impl Memory {
    fn get_type(&self) -> String {
        match self {
            Memory::Model(_) => "Model".to_string(),
            Memory::LazyModel(_) => "LazyModel".to_string(),
            Memory::Block(_, _) => "Block".to_string(),
            Memory::RootBlock(_) => "Block".to_string(),
            Memory::Remap(remap) => format!(
                "Remap({}@{:#016x} -> {:#016x})",
                remap.region.memory.get_type(),
                remap.info.base,
                remap.info.base + remap.info.size
            ),
            Memory::IO(_) => "IO".to_string(),
        }
    }
}

macro_rules! memory_access {
    ($x:ident, $f:ident, $obj:expr, $($p:expr),+) => {match $obj {
            Memory::IO(io) => $x::$f(io.deref(),$($p,)+),
            Memory::Model(model) => $x::$f(model,$($p,)+),
            Memory::LazyModel(model) => $x::$f(model,$($p,)+),
            Memory::Block(_, region) =>  $x::$f(region.deref(),$($p,)+),
            Memory::RootBlock(region) =>  $x::$f(region.deref(),$($p,)+),
            Memory::Remap(remap) => $x::$f(remap.region.deref(),$($p,)+),
        }
        }
}

impl U8Access for Memory {
    fn write(&self, addr: &u64, data: u8) {
        memory_access!(U8Access, write, self, addr, data)
    }

    fn read(&self, addr: &u64) -> u8 {
        memory_access!(U8Access, read, self, addr)
    }
}

impl U16Access for Memory {
    fn write(&self, addr: &u64, data: u16) {
        memory_access!(U16Access, write, self, addr, data)
    }

    fn read(&self, addr: &u64) -> u16 {
        memory_access!(U16Access, read, self, addr)
    }
}

impl U32Access for Memory {
    fn write(&self, addr: &u64, data: u32) {
        memory_access!(U32Access, write, self, addr, data)
    }

    fn read(&self, addr: &u64) -> u32 {
        memory_access!(U32Access, read, self, addr)
    }
}

impl U64Access for Memory {
    fn write(&self, addr: &u64, data: u64) {
        memory_access!(U64Access, write, self, addr, data)
    }

    fn read(&self, addr: &u64) -> u64 {
        memory_access!(U64Access, read, self, addr)
    }
}

impl BytesAccess for Memory {
    fn write(&self, addr: &u64, data: &[u8]) -> Result<usize, String> {
        memory_access!(BytesAccess, write, self, addr, data)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> Result<usize, String> {
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

    pub fn io(base: u64, size: u64, io: Box<dyn IOAccess>) -> Rc<Region> {
        Rc::new(Region {
            memory: Memory::IO(io),
            info: MemInfo {
                base: base,
                size: size,
            },
        })
    }

    fn lazy_model(base: u64, size: u64) -> Region {
        Region {
            memory: Memory::LazyModel(LazyModel::new()),
            info: MemInfo {
                base: base,
                size: size,
            },
        }
    }

    fn model(base: u64, size: u64) -> Region {
        Region {
            memory: Memory::Model(Model::new(MemInfo {
                base: base,
                size: size,
            })),
            info: MemInfo {
                base: base,
                size: size,
            },
        }
    }

    fn block(base: u64, size: u64, heap: &Rc<Heap>, memory: &Rc<Region>) -> Rc<Region> {
        Rc::new(Region {
            memory: Memory::Block(Rc::clone(heap), Rc::clone(memory)),
            info: MemInfo {
                base: base,
                size: size,
            },
        })
    }

    fn root_block(base: u64, size: u64, memory: Region) -> Rc<Region> {
        Rc::new(Region {
            memory: Memory::RootBlock(Box::new(memory)),
            info: MemInfo {
                base: base,
                size: size,
            },
        })
    }

    pub fn remap(base: u64, memory: &Rc<Region>) -> Rc<Region> {
        let info = memory.info;
        Rc::new(Region {
            memory: Memory::Remap(Remap::new(memory, info)),
            info: MemInfo {
                base: base,
                size: info.size,
            },
        })
    }

    pub fn remap_partial(base: u64, memory: &Rc<Region>, offset: u64, size: u64) -> Rc<Region> {
        assert!(offset + size <= memory.info.size);
        assert!(offset & 0x7 == 0);
        let info = memory.info;
        Rc::new(Region {
            memory: Memory::Remap(Remap::new(
                memory,
                MemInfo {
                    base: info.base + offset,
                    size: size,
                },
            )),
            info: MemInfo {
                base: base,
                size: size,
            },
        })
    }
    fn translate(&self, va: &u64, size: usize) -> Option<u64> {
        assert!(
            *va >= self.info.base
                && *va < self.info.base + self.info.size
                && *va + size as u64 - 1 >= self.info.base
                && *va + size as u64 - 1 < self.info.base + self.info.size,
            format!(
                "addr {:#x}-{:#x} translate fail!range {:#x?}",
                *va,
                *va + size as u64 - 1,
                self.info
            )
        );
        match &self.memory {
            Memory::Remap(remap) => Some(va - self.info.base + remap.info.base),
            _ => None,
        }
    }
}

impl U8Access for Region {
    fn write(&self, addr: &u64, data: u8) {
        if let Some(ref a) = self.translate(addr, 1) {
            U8Access::write(&self.memory, a, data)
        } else {
            U8Access::write(&self.memory, addr, data)
        }
    }

    fn read(&self, addr: &u64) -> u8 {
        if let Some(ref a) = self.translate(addr, 1) {
            U8Access::read(&self.memory, a)
        } else {
            U8Access::read(&self.memory, addr)
        }
    }
}

impl BytesAccess for Region {
    fn write(&self, addr: &u64, data: &[u8]) -> Result<usize, String> {
        if let Some(ref a) = self.translate(addr, data.len()) {
            BytesAccess::write(&self.memory, a, data)
        } else {
            BytesAccess::write(&self.memory, addr, data)
        }
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> Result<usize, String> {
        if let Some(ref a) = self.translate(addr, data.len()) {
            BytesAccess::read(&self.memory, a, data)
        } else {
            BytesAccess::read(&self.memory, addr, data)
        }
    }
}

impl U16Access for Region {
    fn write(&self, addr: &u64, data: u16) {
        if let Some(ref a) = self.translate(addr, 2) {
            U16Access::write(&self.memory, a, data)
        } else {
            U16Access::write(&self.memory, addr, data)
        }
    }

    fn read(&self, addr: &u64) -> u16 {
        if let Some(ref a) = self.translate(addr, 2) {
            U16Access::read(&self.memory, a)
        } else {
            U16Access::read(&self.memory, addr)
        }
    }
}

impl U32Access for Region {
    fn write(&self, addr: &u64, data: u32) {
        if let Some(ref a) = self.translate(addr, 4) {
            U32Access::write(&self.memory, a, data)
        } else {
            U32Access::write(&self.memory, addr, data)
        }
    }

    fn read(&self, addr: &u64) -> u32 {
        if let Some(ref a) = self.translate(addr, 4) {
            U32Access::read(&self.memory, a)
        } else {
            U32Access::read(&self.memory, addr)
        }
    }
}

impl U64Access for Region {
    fn write(&self, addr: &u64, data: u64) {
        if let Some(ref a) = self.translate(addr, 8) {
            U64Access::write(&self.memory, a, data)
        } else {
            U64Access::write(&self.memory, addr, data)
        }
    }

    fn read(&self, addr: &u64) -> u64 {
        if let Some(ref a) = self.translate(addr, 8) {
            U64Access::read(&self.memory, a)
        } else {
            U64Access::read(&self.memory, addr)
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
    memory: Rc<Region>,
    pub allocator: RefCell<Allocator>,
}

#[cfg(not(test))]
#[repr(C)]
pub struct Heap {
    memory: Rc<Region>,
    allocator: RefCell<Allocator>,
}

impl Heap {
    pub fn new(memory: &Rc<Region>) -> Rc<Heap> {
        Rc::new(Heap {
            memory: Rc::clone(memory),
            allocator: RefCell::new(Allocator::new(memory.info.base, memory.info.size)),
        })
    }

    pub fn alloc(
        self: &Rc<Self>,
        size: u64,
        align: u64,
    ) -> std::result::Result<Rc<Region>, String> {
        if let Some(info) = self.allocator.borrow_mut().alloc(size, align) {
            Ok(Region::block(info.base, info.size, self, &self.memory))
        } else {
            Err("oom!".to_string())
        }
    }
    pub fn get_region(&self) -> &Rc<Region> {
        &self.memory
    }
}

impl Free for Heap {
    fn free(&self, addr: u64) {
        self.allocator.borrow_mut().free(addr)
    }
}

lazy_static! {
    pub static ref GHEAP: GlobalHeap = GlobalHeap {
        allocator: LockedAllocator::new(0, 0x8000_0000_0000_0000)
    };
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
    pub fn lazy_alloc(&self, size: u64, align: u64) -> std::result::Result<Rc<Region>, String> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Ok(Region::root_block(
                info.base,
                info.size,
                Region::lazy_model(info.base, info.size),
            ))
        } else {
            Err("oom!".to_string())
        }
    }

    pub fn alloc(&self, size: u64, align: u64) -> std::result::Result<Rc<Region>, String> {
        if let Some(info) = self.allocator.alloc(size, align) {
            Ok(Region::root_block(
                info.base,
                info.size,
                Region::model(info.base, info.size),
            ))
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
