use crate::memory::region::Region;
use super::queue::Queue;
use std::sync::Arc;
use crate::irq::{IrqVec, IrqVecSender};

pub struct Device {
    memory: Arc<Region>,
    queues: Vec<Arc<Queue>>,
    irq_vec: Arc<IrqVec>,
    device_id: u32,
    vendor_id: u32,
    device_features: u32,
}

impl Device {
    pub fn new(memory: &Arc<Region>,
               irq_sender: IrqVecSender,
               num_irqs: usize,
               device_id: u32,
               vendor_id: u32,
               device_features: u32) -> Device {
        let irq_vec = IrqVec::new(num_irqs);
        for i in 0..num_irqs {
            let sender = irq_sender.clone();
            irq_vec.binder().bind(i, move |irq_status| {
                irq_status.set_pending(i).unwrap();
                sender.send().unwrap();
            }).unwrap();
        }
        Device {
            memory: Arc::clone(memory),
            queues: vec![],
            irq_vec: Arc::new(irq_vec),
            device_id,
            vendor_id,
            device_features,
        }
    }

    pub fn add_queue(&mut self, queue: Queue) {
        self.queues.push(Arc::new(queue))
    }

    pub fn get_queue(&self, id: usize) -> Arc<Queue> {
        self.queues[id].clone()
    }

    pub fn get_irq_vec(&self) -> Arc<IrqVec> {
        self.irq_vec.clone()
    }
}