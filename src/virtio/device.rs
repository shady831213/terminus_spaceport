use crate::memory::Region;
use super::queue::{Queue,QueueSetting};
use std::rc::Rc;
use std::sync::Arc;

pub struct Device{
    memory:Arc<Region>,
    queues:Vec<Rc<Queue>>,
    device_id:u32,
    vendor_id:u32,
    device_features:u32,
}

impl Device {
    pub fn new(memory:&Arc<Region>,
               queue_settings:&[QueueSetting],
               device_id:u32,
               vendor_id:u32,
               device_features:u32) -> Device {

        let mut device = Device {
            memory:Arc::clone(memory),
            queues:vec![],
            device_id,
            vendor_id,
            device_features
        };
        for &s in queue_settings {
            device.queues.push(Rc::new(Queue::new(memory, s)))
        }
        device
    }
}