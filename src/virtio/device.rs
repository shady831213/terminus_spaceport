use crate::memory::region::Region;
use super::queue::Queue;
use crate::irq::{IrqVec, IrqVecSender};
use std::rc::Rc;
use std::cell::RefCell;
use crate::virtio::{MAX_QUEUE,
                    MAX_QUEUE_NUM,
                    MMIO_MAGIC_VALUE,
                    MMIO_VERSION,
                    MMIO_DEVICE_ID,
                    MMIO_VENDOR_ID,
                    MMIO_DEVICE_FEATURES,
                    MMIO_DRIVER_FEATURES,
                    MMIO_QUEUE_SEL,
                    MMIO_QUEUE_NUM_MAX,
                    MMIO_QUEUE_NUM,
                    MMIO_QUEUE_READY,
                    MMIO_QUEUE_NOTIFY,
                    MMIO_INTERRUPT_STATUS,
                    MMIO_INTERRUPT_ACK,
                    MMIO_STATUS,
                    MMIO_QUEUE_DESC_LOW,
                    MMIO_QUEUE_DESC_HIGH,
                    MMIO_QUEUE_AVAIL_LOW,
                    MMIO_QUEUE_AVAIL_HIGH,
                    MMIO_QUEUE_USED_LOW,
                    MMIO_QUEUE_USED_HIGH,
                    MMIO_CONFIG};

pub struct Device {
    memory: Rc<Region>,
    queues: Vec<Queue>,
    irq_sender: Option<IrqVecSender>,
    irq_vec: IrqVec,
    queue_sel: RefCell<u32>,
    status: RefCell<u32>,
    device_id: u32,
    vendor_id: u32,
    device_features: u32,
}

impl Device {
    pub fn new(memory: &Rc<Region>,
               irq_sender: Option<IrqVecSender>,
               num_irqs: usize,
               device_id: u32,
               vendor_id: u32,
               device_features: u32) -> Device {
        let mut irq_vec = IrqVec::new(num_irqs);
        if let Some(ref sender) = irq_sender {
            for i in 0..num_irqs {
                let s = sender.clone();
                irq_vec.binder().bind(i, move || {
                    s.send().unwrap();
                }).unwrap();
            }
        } else {
            assert_eq!(num_irqs, 0)
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
        if let Some(ref sender) = self.irq_sender {
            sender.clear().unwrap();
        }
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

    pub fn memory(&self) -> &Rc<Region> {
        &self.memory
    }
}

pub trait DeviceAccess {
    fn device(&self) -> &Device;

    fn magic(&self) -> u32 { 0x74726976 }

    fn version(&self) -> u32 { 2 }

    fn device_id(&self) -> u32 { self.device().device_id }

    fn vendor_id(&self) -> u32 { self.device().vendor_id }

    fn device_features(&self) -> u32 { self.device().device_features }

    fn queue_sel(&self) -> u32 { *self.device().queue_sel.borrow() }

    fn set_queue_sel(&self, val: &u32) {
        assert!((*val as usize) < self.device().queues.len());
        *self.device().queue_sel.borrow_mut() = *val
    }

    fn queue_num_max(&self) -> u32 { MAX_QUEUE_NUM as u32 }

    fn queue_num(&self) -> u32 { self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_queue_size() as u32 }

    fn set_queue_num(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_queue_size(*val as u16) }

    fn queue_desc_low(&self) -> u32 { self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_desc_addr() as u32 }

    fn set_queue_desc_low(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_desc_addr_low(*val) }

    fn queue_avail_low(&self) -> u32 { self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_avail_addr() as u32 }

    fn set_queue_avail_low(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_avail_addr_low(*val) }

    fn queue_used_low(&self) -> u32 { self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_used_addr() as u32 }

    fn set_queue_used_low(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_used_addr_low(*val) }

    fn queue_desc_high(&self) -> u32 { (self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_desc_addr() >> 32) as u32 }

    fn set_queue_desc_high(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_desc_addr_high(*val) }

    fn queue_avail_high(&self) -> u32 { (self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_avail_addr() >> 32) as u32 }

    fn set_queue_avail_high(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_avail_addr_high(*val) }

    fn queue_used_high(&self) -> u32 { (self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_used_addr() >> 32) as u32 }

    fn set_queue_used_high(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_used_addr_high(*val) }

    fn queue_ready(&self) -> u32 { self.device().get_queue(*self.device().queue_sel.borrow() as usize).get_ready() as u32 }

    fn set_queue_ready(&self, val: &u32) { self.device().get_queue(*self.device().queue_sel.borrow() as usize).set_ready(*val & 0x1 == 0x1) }

    fn int_status(&self) -> u32 {
        let irq = self.device().get_irq_vec();
        irq.pendings() as u32
    }

    fn int_ack(&self, val: &u32) {
        self.device().irq_vec.clr_pendings(*val as u64);
        if self.device().irq_vec.pendings() == 0 {
            if let Some(ref sender) = self.device().irq_sender {
                sender.clear().unwrap();
            }
        }
    }

    fn status(&self) -> u32 {
        *self.device().status.borrow()
    }

    fn set_status(&self, val: &u32) {
        if *val == 0 {
            self.device().reset()
        }
        *self.device().status.borrow_mut() = *val;
    }

    fn config(&self, _: u64) -> u32 { 0 }

    fn set_config(&self, _: u64, _: &u32) {}

    fn queue_notify(&self, val: &u32) {
        let id = *val as usize;
        if id < self.device().queues.len() {
            self.device().get_queue(id).notify_client().unwrap()
        }
    }
}

pub trait MMIODevice: DeviceAccess {
    fn read_bytes(&self, offset: &u64, data: &mut [u8]) {
        if data.len() == 4 {
            data.copy_from_slice(&self.read_u32(offset).to_le_bytes())
        }
    }

    fn write_bytes(&self, offset: &u64, data: &[u8]) {
        if data.len() == 4 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(data);
            self.write_u32(offset, &u32::from_le_bytes(bytes))
        }
    }

    fn read_u32(&self, offset: &u64) -> u32 {
        if *offset >= MMIO_CONFIG {
            return self.config(*offset - MMIO_CONFIG);
        }
        match *offset {
            MMIO_MAGIC_VALUE => self.magic(),
            MMIO_VERSION => self.version(),
            MMIO_DEVICE_ID => self.device_id(),
            MMIO_VENDOR_ID => self.vendor_id(),
            MMIO_DEVICE_FEATURES => self.device_features(),
            MMIO_QUEUE_SEL => self.queue_sel(),
            MMIO_QUEUE_NUM_MAX => self.queue_num_max(),
            MMIO_QUEUE_NUM => self.queue_num(),
            MMIO_QUEUE_DESC_LOW => self.queue_desc_low(),
            MMIO_QUEUE_AVAIL_LOW => self.queue_avail_low(),
            MMIO_QUEUE_USED_LOW => self.queue_used_low(),
            MMIO_QUEUE_DESC_HIGH => self.queue_desc_high(),
            MMIO_QUEUE_AVAIL_HIGH => self.queue_avail_high(),
            MMIO_QUEUE_USED_HIGH => self.queue_used_high(),
            MMIO_QUEUE_READY => self.queue_ready(),
            MMIO_INTERRUPT_STATUS => self.int_status(),
            MMIO_STATUS => self.status(),
            _ => 0
        }
    }

    fn write_u32(&self, offset: &u64, val: &u32) {
        if *offset >= MMIO_CONFIG {
            return self.set_config(*offset - MMIO_CONFIG, val);
        }
        match *offset {
            MMIO_QUEUE_SEL => self.set_queue_sel(val),
            MMIO_QUEUE_NUM => self.set_queue_num(val),
            MMIO_QUEUE_DESC_LOW => self.set_queue_desc_low(val),
            MMIO_QUEUE_AVAIL_LOW => self.set_queue_avail_low(val),
            MMIO_QUEUE_USED_LOW => self.set_queue_used_low(val),
            MMIO_QUEUE_DESC_HIGH => self.set_queue_desc_high(val),
            MMIO_QUEUE_AVAIL_HIGH => self.set_queue_avail_high(val),
            MMIO_QUEUE_USED_HIGH => self.set_queue_used_high(val),
            MMIO_QUEUE_READY => self.set_queue_ready(val),
            MMIO_STATUS => self.set_status(val),
            MMIO_QUEUE_NOTIFY => self.queue_notify(val),
            MMIO_INTERRUPT_ACK => self.int_ack(val),
            _ => {}
        }
    }
}

