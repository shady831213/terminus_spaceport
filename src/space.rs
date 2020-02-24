use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use crate::model::*;
use std::ops::Deref;

//Space should be an owner of Regions
pub struct Space {
    regions: Mutex<HashMap<String, Arc<Region>>>
}

impl Space {
    pub fn new() -> Space {
        Space { regions: Mutex::new(HashMap::new()) }
    }

    pub fn add_region(&self, name: String, region: &Arc<Region>) -> Arc<Region> {
        let mut map = self.regions.lock().unwrap();
        let check = || {
            if let Some(_) = map.get(&name) {
                panic!("region name {} has existed!", name);
            }
            if let Some(v) = map.iter().find(|(_, v)| {
                region.info.base >= v.info.base && region.info.base < v.info.base + v.info.size ||
                    region.info.base + region.info.size - 1 >= v.info.base && region.info.base + region.info.size - 1 < v.info.base + v.info.size
            }) {
                panic!("region [{} : {:?}] is overlapped with [{} : {:?}]!", name, region.deref().info, v.0, v.1.deref().info);
            }
        };
        check();
        map.insert(name, Arc::clone(region));
        Arc::clone(region)
    }

    pub fn delete_region(&self, name: String) {
        if let Some(v) = self.regions.lock().unwrap().remove(&name) {
            std::mem::drop(v)
        }
    }

    pub fn get_region(&self, name: String) -> Arc<Region> {
        if let Some(v) = self.regions.lock().unwrap().get(&name) {
            Arc::clone(v)
        } else {
            panic!(format!("no region {}!", name))
        }
    }

    pub fn get_region_by_addr(&self, addr: u64) -> Arc<Region> {
        if let Some(v) = self.regions.lock().unwrap().values().find(|v| { addr >= v.info.base && addr < v.info.base + v.info.size }) {
            Arc::clone(v)
        } else {
            panic!(format!("addr 0x{:x?} is not valid for any region!", addr))
        }
    }
}
