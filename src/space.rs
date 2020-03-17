use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use crate::model::*;
use std::ops::Deref;

#[derive(Debug)]
pub enum Error {
    Overlap(String, String),
    Renamed(String, String),
}

//Space should be an owner of Regions
pub struct Space {
    regions: HashMap<String, Arc<Region>>,
    //for ffi free
    ptrs: HashMap<String, Vec<*const Box<Arc<Region>>>>,
}

impl Space {
    pub fn new() -> Space {
        Space { regions: HashMap::new(), ptrs: HashMap::new() }
    }

    pub fn add_region(&mut self, name: &str, region: &Arc<Region>) -> Result<Arc<Region>, Error> {
        let check = || {
            if let Some(v) = self.regions.get(name) {
                return Err(Error::Renamed(name.to_string(), format!("region name {} has existed!", name)));
            }
            if let Some(v) = self.regions.iter().find(|(_, v)| {
                region.info.base >= v.info.base && region.info.base < v.info.base + v.info.size ||
                    region.info.base + region.info.size - 1 >= v.info.base && region.info.base + region.info.size - 1 < v.info.base + v.info.size
            }) {
                return Err(Error::Overlap(v.0.to_string(), format!("region [{} : {:?}] is overlapped with [{} : {:?}]!", name, region.deref().info, v.0, v.1.deref().info)));
            }
            Ok(())
        };
        check()?;
        self.regions.insert(String::from(name), Arc::clone(region));
        Ok(Arc::clone(region))
    }

    pub fn delete_region(&mut self, name: &str) {
        if let Some(v) = self.regions.remove(name) {
            std::mem::drop(v)
        }
        if let Some(ptrs) = self.ptrs.remove(name) {
            ptrs.iter().for_each(|ptr| { std::mem::drop(unsafe { (*ptr).read() }) })
        }
    }

    pub fn get_region(&self, name: &str) -> Option<Arc<Region>> {
        if let Some(v) = self.regions.get(name) {
            Some(Arc::clone(v))
        } else {
            None
        }
    }

    pub fn get_region_by_addr(&self, addr: u64) -> Option<Arc<Region>> {
        if let Some(v) = self.regions.values().find(|v| { addr >= v.info.base && addr < v.info.base + v.info.size }) {
            Some(Arc::clone(v))
        } else {
            None
        }
    }

    pub fn clean(&mut self, name: &str, ptr: *const Box<Arc<Region>>) {
        let e = self.ptrs.entry(String::from(name)).or_insert(vec![]);
        e.push(ptr)
    }
}

pub struct SpaceTable {
    spaces: Mutex<HashMap<String, Arc<RwLock<Space>>>>,
}

impl SpaceTable {
    pub fn global() -> Arc<SpaceTable> {
        static mut SPACE_TABLE: Option<Arc<SpaceTable>> = None;

        unsafe {
            SPACE_TABLE.get_or_insert_with(|| {
                Arc::new(SpaceTable { spaces: Mutex::new(HashMap::new()) })
            }).clone()
        }
    }

    pub fn get_space(&self, name: &str) -> Arc<RwLock<Space>> {
        Arc::clone(self.spaces.lock().unwrap()
            .entry(String::from(name))
            .or_insert(Arc::new(RwLock::new(Space::new()))))
    }
}
