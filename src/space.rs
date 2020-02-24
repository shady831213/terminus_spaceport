use std::collections::HashMap;
use std::sync::Arc;
use crate::model::*;
use std::ops::Deref;

//Space should be an owner of Regions
pub struct Space {
    regions: HashMap<String, Arc<Region>>
}

impl Space {
    pub fn new() -> Space {
        Space { regions: HashMap::new() }
    }

    pub fn add_region(&mut self, name: String, region: &Arc<Region>) {
        let check = || {
            if let Some(_) = self.regions.get(&name) {
                panic!("region name {} has existed!", name);
            }
            if let Some(v) = self.regions.iter().find(|(_, v)| {
                region.info.base >= v.info.base && region.info.base < v.info.base + v.info.size ||
                    region.info.base + region.info.size - 1 >= v.info.base && region.info.base + region.info.size - 1 < v.info.base + v.info.size
            }) {
                panic!("region [{} : {:?}] is overlapped with [{} : {:?}]!", name, region.deref().info, v.0, v.1.deref().info);
            }
        };
        check();
        self.regions.insert(name, Arc::clone(region));
    }

    pub fn delete_region(&mut self, name: String) {
        if let Some(v) = self.regions.remove(&name) {
            std::mem::drop(v)
        }
    }

    pub fn get_region(&self, name: String) -> Option<&Arc<Region>> {
        self.regions.get(&name)
    }

    pub fn get_region_by_addr(&self, addr: u64) -> Option<&Arc<Region>> {
        self.regions.values().find(|v| { addr >= v.info.base && addr < v.info.base + v.info.size })
    }
}
