use crate::memory::region::Region;
use super::queue::Queue;
use crate::irq::{IrqVec, IrqVecSender};
use std::rc::Rc;

pub struct Device {
    memory: Rc<Region>,
    queues: Vec<Rc<Queue>>,
    irq_vec: Rc<IrqVec>,
    device_id: u32,
    vendor_id: u32,
    device_features: u32,
}

impl Device {
    pub fn new(memory: &Rc<Region>,
               irq_sender: IrqVecSender,
               num_irqs: usize,
               device_id: u32,
               vendor_id: u32,
               device_features: u32) -> Device {
        let irq_vec = IrqVec::new(num_irqs);
        for i in 0..num_irqs {
            let sender = irq_sender.clone();
            irq_vec.binder().bind(i, move || {
                sender.send().unwrap();
            }).unwrap();
        }
        Device {
            memory: Rc::clone(memory),
            queues: vec![],
            irq_vec: Rc::new(irq_vec),
            device_id,
            vendor_id,
            device_features,
        }
    }

    pub fn add_queue(&mut self, queue: Queue) {
        self.queues.push(Rc::new(queue))
    }

    pub fn get_queue(&self, id: usize) -> Rc<Queue> {
        self.queues[id].clone()
    }

    pub fn get_irq_vec(&self) -> Rc<IrqVec> {
        self.irq_vec.clone()
    }
}