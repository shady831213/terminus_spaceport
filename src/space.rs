extern crate intrusive_collections;

use std::collections::HashMap;
use crate::memory::region::{Region, U8Access, U16Access, U32Access, U64Access, BytesAccess};
use std::ops::Deref;
use std::fmt::{Display, Formatter};
use std::fmt;
use intrusive_collections::{RBTreeLink, KeyAdapter, intrusive_adapter, Bound};
use intrusive_collections::rbtree::RBTree;
use std::rc::Rc;

struct SpaceElem {
    link: RBTreeLink,
    key: u64,
    value: (String, Rc<Region>),
}

intrusive_adapter!(Adapter = Box<SpaceElem>:SpaceElem {link:RBTreeLink});

impl<'a> KeyAdapter<'a> for Adapter {
    type Key = &'a u64;
    fn get_key(&self, s: &'a SpaceElem) -> &'a u64 { &s.key }
}

#[derive(Debug)]
pub enum Error {
    Overlap(String, String),
    Renamed(String, String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Overlap(s1, s2) => writeln!(f, "{}:{}", s1, s2),
            Error::Renamed(s1, s2) => writeln!(f, "{}:{}", s1, s2)
        }
    }
}
struct RegionCPtr(*const Box<Rc<Region>>);

unsafe impl Send for RegionCPtr {}

unsafe impl Sync for RegionCPtr {}


//Space should be an owner of Regions
pub struct Space {
    regions: RBTree<Adapter>,
    //for ffi free
    ptrs: HashMap<String, Vec<RegionCPtr>>,
}

impl Space {
    pub fn new() -> Space {
        Space { regions: RBTree::new(Adapter::default()), ptrs: HashMap::new() }
    }

    pub fn add_region(&mut self, name: &str, region: &Rc<Region>) -> Result<Rc<Region>, Error> {
        let check = || {
            if let Some(_) = self.regions.iter().find(|a| { a.value.0 == name }) {
                return Err(Error::Renamed(name.to_string(), format!("region name {} has existed!", name)));
            }
            if let Some(v) = self.regions.iter().find(|a| {
                region.info.base >= a.value.1.info.base && region.info.base < a.value.1.info.base + a.value.1.info.size ||
                    region.info.base + region.info.size - 1 >= a.value.1.info.base && region.info.base + region.info.size - 1 < a.value.1.info.base + a.value.1.info.size ||
                    a.value.1.info.base >= region.info.base && a.value.1.info.base < region.info.base + region.info.size ||
                    a.value.1.info.base + a.value.1.info.size - 1 >= region.info.base && a.value.1.info.base + a.value.1.info.size - 1 < region.info.base + region.info.size
            }) {
                return Err(Error::Overlap(v.value.0.to_string(), format!("region [{} : {:?}] is overlapped with [{} : {:?}]!", name, region.deref().info, v.value.0, v.value.1.deref().info)));
            }
            Ok(())
        };
        check()?;
        // self.regions.insert(region.info.base, (name.to_string(), Rc::clone(region)));
        self.regions.insert(Box::new(SpaceElem { link: RBTreeLink::new(), key: region.info.base, value: (name.to_string(), Rc::clone(region)) }));
        Ok(Rc::clone(region))
    }

    pub fn delete_region(&mut self, name: &str) {
        let mut cursor = self.regions.front_mut();
        while !cursor.is_null() {
            if let Some(e) = cursor.get() {
                if e.value.0 == name {
                    cursor.remove();
                    break
                }
            }
            cursor.move_next();
        }
        if let Some(ps) = self.ptrs.remove(name) {
            ps.iter().for_each(|RegionCPtr(ptr)| { std::mem::drop(unsafe { (*ptr).read() }) })
        }
    }

    pub fn get_region(&self, name: &str) -> Option<Rc<Region>> {
        if let Some(v) = self.regions.iter().find_map(|a| { if a.value.0 == name { Some(&a.value.1) } else { None } }) {
            Some(Rc::clone(v))
        } else {
            None
        }
    }

    pub fn get_region_by_addr(&self, addr: &u64) -> Result<Rc<Region>, u64> {
        if let Some(e) = self.regions.upper_bound(Bound::Included(addr)).get(){
            if *addr < e.value.1.info.base + e.value.1.info.size {
                Ok(Rc::clone(&e.value.1))
            } else {
                Err(*addr)
            }
        } else {
            Err(*addr)
        }
    }

    pub fn write_u8(&self, addr: &u64, data: u8) -> Result<(), u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U8Access::write(region.deref(), addr, data))
    }

    pub fn read_u8(&self, addr: &u64) -> Result<u8, u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U8Access::read(region.deref(), addr))
    }

    pub fn write_u16(&self, addr: &u64, data: u16) -> Result<(), u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U16Access::write(region.deref(), addr, data))
    }

    pub fn read_u16(&self, addr: &u64) -> Result<u16, u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U16Access::read(region.deref(), addr))
    }

    pub fn write_u32(&self, addr: &u64, data: u32) -> Result<(), u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U32Access::write(region.deref(), addr, data))
    }

    pub fn read_u32(&self, addr: &u64) -> Result<u32, u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U32Access::read(region.deref(), addr))
    }

    pub fn write_u64(&self, addr: &u64, data: u64) -> Result<(), u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U64Access::write(region.deref(), addr, data))
    }

    pub fn read_u64(&self, addr: &u64) -> Result<u64, u64> {
        let region = self.get_region_by_addr(addr)?;
        Ok(U64Access::read(region.deref(), addr))
    }

    pub fn write_bytes(&self, addr: &u64, data: &[u8]) -> Result<usize, u64> {
        let region = self.get_region_by_addr(addr)?;
        if let Ok(size) = BytesAccess::write(region.deref(), addr, data) {
            Ok(size)
        } else {
            Err(*addr)
        }
    }

    pub fn read_bytes(&self, addr: &u64, data: &mut [u8]) -> Result<usize, u64> {
        let region = self.get_region_by_addr(addr)?;
        if let Ok(size) = BytesAccess::read(region.deref(), addr, data) {
            Ok(size)
        } else {
            Err(*addr)
        }
    }

    pub fn clean(&mut self, name: &str, ptr: *const Box<Rc<Region>>) {
        self.ptrs.entry(String::from(name)).or_insert(vec![])
            .push(RegionCPtr(ptr))
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "regions:")?;
        for e in self.regions.iter() {
            writeln!(f, "   {:<10}({:^13})  : {:#016x} -> {:#016x}", e.value.0, e.value.1.get_type(), e.value.1.info.base, e.value.1.info.base + e.value.1.info.size - 1)?;
        }
        Ok(())
    }
}
