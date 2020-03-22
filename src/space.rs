use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::memory::*;
use std::ops::Deref;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Overlap(String, String),
    Renamed(String, String),
}


struct RegionCPtr(*const Box<Arc<Region>>);

unsafe impl Send for RegionCPtr {}

unsafe impl Sync for RegionCPtr {}


//Space should be an owner of Regions
pub struct Space {
    regions: Mutex<HashMap<String, Arc<Region>>>,
    //for ffi free
    ptrs: Mutex<HashMap<String, Vec<RegionCPtr>>>,
}

impl Space {
    pub fn new() -> Space {
        Space { regions: Mutex::new(HashMap::new()), ptrs: Mutex::new(HashMap::new()) }
    }

    pub fn add_region(&self, name: &str, region: &Arc<Region>) -> Result<Arc<Region>, Error> {
        let mut map = self.regions.lock().unwrap();
        let check = || {
            if let Some(_) = map.get(name) {
                return Err(Error::Renamed(name.to_string(), format!("region name {} has existed!", name)));
            }
            if let Some(v) = map.iter().find(|(_, v)| {
                region.info.base >= v.info.base && region.info.base < v.info.base + v.info.size ||
                    region.info.base + region.info.size - 1 >= v.info.base && region.info.base + region.info.size - 1 < v.info.base + v.info.size ||
                    v.info.base >= region.info.base && v.info.base < region.info.base + region.info.size ||
                    v.info.base + v.info.size - 1 >= region.info.base && v.info.base + v.info.size - 1 < region.info.base + region.info.size
            }) {
                return Err(Error::Overlap(v.0.to_string(), format!("region [{} : {:?}] is overlapped with [{} : {:?}]!", name, region.deref().info, v.0, v.1.deref().info)));
            }
            Ok(())
        };
        check()?;
        map.insert(String::from(name), Arc::clone(region));
        Ok(Arc::clone(region))
    }

    pub fn delete_region(&self, name: &str) {
        self.regions.lock().unwrap().remove(name);
        let mut ptrs = self.ptrs.lock().unwrap();
        if let Some(ps) = ptrs.remove(name) {
            ps.iter().for_each(|RegionCPtr(ptr)| { std::mem::drop(unsafe { (*ptr).read() }) })
        }
    }

    pub fn get_region(&self, name: &str) -> Option<Arc<Region>> {
        let map = self.regions.lock().unwrap();
        if let Some(v) = map.get(name) {
            Some(Arc::clone(v))
        } else {
            None
        }
    }

    pub fn get_region_by_addr(&self, addr: u64) -> Option<Arc<Region>> {
        let map = self.regions.lock().unwrap();
        if let Some(v) = map.values().find(|v| { addr >= v.info.base && addr < v.info.base + v.info.size }) {
            Some(Arc::clone(v))
        } else {
            None
        }
    }

    pub fn clean(&self, name: &str, ptr: *const Box<Arc<Region>>) {
        self.ptrs.lock().unwrap()
            .entry(String::from(name)).or_insert(vec![])
            .push(RegionCPtr(ptr))
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut map = self.regions.lock().unwrap();
        let mut paires = map.iter_mut().collect::<Vec<_>>();
        paires.sort_by(|l, r| { l.1.info.base.cmp(&r.1.info.base) });
        writeln!(f, "regions:")?;
        for (name, region) in paires {
            writeln!(f, "   {:<10}({:^13})  : {:#016x} -> {:#016x}", name, region.get_type(), region.info.base, region.info.base + region.info.size - 1)?;
        }
        Ok(())
    }
}


lazy_static! {
    pub static ref SPACE_TABLE:SpaceTable = SpaceTable { spaces: Mutex::new(HashMap::new()) };
}

pub struct SpaceTable {
    spaces: Mutex<HashMap<String, Arc<Space>>>,
}

impl SpaceTable {
    pub fn get_space(&self, name: &str) -> Arc<Space> {
        let mut map = self.spaces.lock().unwrap();
        map.entry(String::from(name))
            .or_insert_with(|| {
                if name == "space_query" {
                    println!("create space_query")
                }
                Arc::new(Space::new())
            }).clone()
    }
}
