use crate::memory::region::Region;
use super::queue::Queue;
use crate::irq::{IrqVec, IrqVecSender};
use std::rc::Rc;
use std::cell::RefCell;
use crate::virtio::{MAX_QUEUE, MAX_QUEUE_NUM};

pub struct Device {
    memory: Rc<Region>,
    queues: Vec<Queue>,
    irq_sender: IrqVecSender,
    irq_vec: IrqVec,
    queue_sel: RefCell<u32>,
    status: RefCell<u32>,
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
        let mut irq_vec = IrqVec::new(num_irqs);
        for i in 0..num_irqs {
            let sender = irq_sender.clone();
            irq_vec.binder().bind(i, move || {
                sender.send().unwrap();
            }).unwrap();
        }
        Device {
            memory: Rc::clone(memory),
            queues: vec![],
            irq_sender,
            irq_vec: irq_vec,
            queue_sel: RefCell::new(0),
            status: RefCell::new(0),
            device_id,
            vendor_id,
            device_features,
        }
    }

    pub fn reset(&self) {
        self.irq_sender.clear().unwrap();
        *self.queue_sel.borrow_mut() = 0;
        *self.status.borrow_mut() = 0;
        for q in self.queues.iter() {
            q.reset()
        }
    }

    pub fn add_queue(&mut self, queue: Queue) {
        self.queues.push(queue);
        assert!(self.queues.len() < (MAX_QUEUE as usize));
    }

    pub fn get_queue(&self, id: usize) -> &Queue {
        unsafe { self.queues.get_unchecked(id) }
    }

    pub fn get_irq_vec(&self) -> &IrqVec {
        &self.irq_vec
    }
}

pub trait DeviceAccess {
    fn magic(&self) -> u32 { 0x74726976 }

    fn version(&self) -> u32 { 2 }

    fn device_id(&self, d: &Device) -> u32 { d.device_id }

    fn vendor_id(&self, d: &Device) -> u32 { d.vendor_id }

    fn device_features(&self, d: &Device) -> u32 { d.device_features }

    fn queue_sel(&self, d: &Device) -> u32 { *d.queue_sel.borrow() }

    fn set_queue_sel(&self, d: &Device, val: &u32) {
        assert!((*val as usize) < d.queues.len());
        *d.queue_sel.borrow_mut() = *val
    }

    fn max_queue_num(&self) -> u32 { MAX_QUEUE_NUM as u32 }

    fn queue_num(&self, d: &Device) -> u32 { d.get_queue(*d.queue_sel.borrow() as usize).get_queue_size() as u32 }

    fn set_queue_num(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_queue_size(*val as u16) }

    fn queue_desc_low(&self, d: &Device) -> u32 { d.get_queue(*d.queue_sel.borrow() as usize).get_desc_addr() as u32 }

    fn set_queue_desc_low(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_desc_addr_low(*val) }

    fn queue_avail_low(&self, d: &Device) -> u32 { d.get_queue(*d.queue_sel.borrow() as usize).get_avail_addr() as u32 }

    fn set_queue_avail_low(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_avail_addr_low(*val) }

    fn queue_used_low(&self, d: &Device) -> u32 { d.get_queue(*d.queue_sel.borrow() as usize).get_used_addr() as u32 }

    fn set_queue_used_low(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_used_addr_low(*val) }

    fn queue_desc_high(&self, d: &Device) -> u32 { (d.get_queue(*d.queue_sel.borrow() as usize).get_desc_addr() >> 32) as u32 }

    fn set_queue_desc_high(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_desc_addr_high(*val) }

    fn queue_avail_high(&self, d: &Device) -> u32 { (d.get_queue(*d.queue_sel.borrow() as usize).get_avail_addr() >> 32) as u32 }

    fn set_queue_avail_high(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_avail_addr_high(*val) }

    fn queue_used_high(&self, d: &Device) -> u32 { (d.get_queue(*d.queue_sel.borrow() as usize).get_used_addr() >> 32) as u32 }

    fn set_queue_used_high(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_used_addr_high(*val) }

    fn queue_ready(&self, d: &Device) -> u32 { d.get_queue(*d.queue_sel.borrow() as usize).get_ready() as u32 }

    fn set_queue_ready(&self, d: &Device, val: &u32) { d.get_queue(*d.queue_sel.borrow() as usize).set_ready(*val & 0x1 == 0x1) }

    fn int_status(&self, d: &Device) -> u32 {
        let irq = d.get_irq_vec();
        irq.pendings() as u32
    }

    fn int_ack(&self, d: &Device, val: &u32) {
        d.irq_vec.clr_pendings(*val as u64);
        if d.irq_vec.pendings() == 0 {
            d.irq_sender.clear().unwrap();
        }
    }

    fn status(&self, d: &Device) -> u32 {
        *d.status.borrow()
    }

    fn set_status(&self, d: &Device, val: &u32) {
        if *val == 0 {
            d.reset()
        }
        *d.status.borrow_mut() = *val;
    }

    fn config(&self, _: u64) -> u32 { 0 }

    fn queue_notify(&self, d: &Device, val: &u32) {
        let id = *val as usize;
        if id < d.queues.len() {
            d.get_queue(id).notify_client().unwrap()
        }
    }
}