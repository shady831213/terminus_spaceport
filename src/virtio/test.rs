use super::queue::{Queue, QueueSetting, DescMeta, RingUsedMetaElem, QueueClient, DESC_F_WRITE, DefaultQueueServer, QueueServer, RingAvailMetaElem, RingMetaHeader, RingMeta};
use std::sync::Arc;
use std::mem;
use crate::memory::{Region, BytesAccess, GHEAP, Heap, U32Access};
use std::ops::Deref;
use crate::irq::{IrqVec, IrqVecSender};
use super::device::Device;
use std::cell::RefCell;

struct TestQueueClient {
    memory: Arc<Region>,
    irq_sender: IrqVecSender,
}

impl TestQueueClient {
    fn new(memory: &Arc<Region>, irq_sender: IrqVecSender) -> TestQueueClient {
        TestQueueClient {
            memory: Arc::clone(memory),
            irq_sender,
        }
    }
}

impl QueueClient for TestQueueClient {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        let writes = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE != 0 {
                    let mut data = vec![0 as u8; desc.len as usize];
                    BytesAccess::read(self.memory.deref(), desc.addr, &mut data);
                    data.iter_mut().for_each(|d| { *d = !*d });
                    Some(data)
                } else {
                    None
                }
            }).collect::<Vec<_>>();

        let count = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE == 0 {
                    Some(desc)
                } else {
                    None
                }
            })
            .enumerate()
            .map(|(i, desc)| {
                BytesAccess::write(self.memory.deref(), desc.addr, &writes[i]);
                writes[i].len() * 2
            })
            .fold(0, |acc, c| { acc + c });
        queue.set_used(desc_head, count as u32)?;
        self.irq_sender.send().unwrap();
        Ok(true)
    }
}

struct TestQueueServer(DefaultQueueServer);

impl TestQueueServer {
    fn irq_handler(&self, memory: &Region, queue: &Queue) {
        let used = self.pop_used(queue).unwrap();
        for desc_res in queue.desc_iter(used.id as u16) {
            let (idx, desc) = desc_res.unwrap();
            if desc.flags & DESC_F_WRITE == 0 {
                assert_eq!(U32Access::read(memory, desc.addr), !(0xdeadbeaf as u32))
            } else {
                assert_eq!(U32Access::read(memory, desc.addr), 0xdeadbeaf)
            }
            println!("desc:{}, data:{:#x}", idx, U32Access::read(memory, desc.addr))
        }
        self.free_used(queue, &used).unwrap();
        assert_eq!(used, RingUsedMetaElem { id: 0, len: 8 })
    }
}

impl Deref for TestQueueServer {
    type Target = DefaultQueueServer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[test]
fn queue_basic_test() {
    const QUEUE_SIZE: usize = 10;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let irq = IrqVec::new(2);
    irq.set_enable(0).unwrap();
    irq.set_enable(1).unwrap();
    let client = TestQueueClient::new(&memory, irq.sender(0).unwrap());

    let queue = Arc::new(Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16 }, client));


    let heap = Heap::new(&memory);
    let desc_mem = heap.alloc(mem::size_of::<DescMeta>() as u64 * queue.get_queue_size() as u64, 4).unwrap();
    let avail_ring: RingMeta<[RingAvailMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader::empty(),
        ring: [0 as RingAvailMetaElem; QUEUE_SIZE],
    };
    let avail_mem = heap.alloc(mem::size_of_val(&avail_ring) as u64, 2).unwrap();
    let used_ring: RingMeta<[RingUsedMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader::empty(),
        ring: [RingUsedMetaElem::empty(); QUEUE_SIZE],
    };
    let used_mem = heap.alloc(mem::size_of_val(&used_ring) as u64, 2).unwrap();
    queue.set_desc_addr(desc_mem.info.base);
    queue.set_avail_addr(avail_mem.info.base);
    queue.set_used_addr(used_mem.info.base);


    let server = Arc::new(TestQueueServer(DefaultQueueServer::new()));
    server.init_queue(&queue).unwrap();
    // server.bind_irq(irq.binder(), &memory, &queue);

    let irq_cnt = Arc::new(RefCell::new(0));
    irq.binder().bind(0, {
        let server_ref = Arc::clone(&server);
        let mem_ref = Arc::clone(&memory);
        let queue_ref = Arc::clone(&queue);
        let mut _cnt = Arc::clone(&irq_cnt);
        move |_| {
            server_ref.irq_handler(mem_ref.deref(), queue_ref.deref());
            *_cnt.deref().borrow_mut() += 1;
        }
    }).unwrap();


    let read_mem = heap.alloc(4, 4).unwrap();
    let write_mem = heap.alloc(4, 4).unwrap();
    U32Access::write(write_mem.deref(), write_mem.info.base, 0xdeadbeaf);
    server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    assert_eq!(*irq_cnt.deref().borrow(), 4);
}

struct TestDevice {
    virtio_device: Device,
}

impl TestDevice {
    pub fn new(memory: &Arc<Region>, irq_sender: IrqVecSender) -> TestDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            1,
                                            0, 0, 0,
        );
        virtio_device.get_irq_vec().set_enable(0).unwrap();
        let input_queue = {
            let input = TestDeviceInput::new(memory, virtio_device.get_irq_vec().sender(0).unwrap());
            Queue::new(&memory, QueueSetting { max_queue_size: 1 },input)
        };
        let output_queue = {
            let output = TestDeviceOutput::new(memory);
            Queue::new(&memory, QueueSetting { max_queue_size: 1 },output)
        };
        virtio_device.add_queue(input_queue);
        virtio_device.add_queue(output_queue);

        TestDevice {
            virtio_device,
        }
    }
}

struct TestDeviceInput {
    memory: Arc<Region>,
    irq_sender: IrqVecSender,
}

impl TestDeviceInput {
    fn new(memory: &Arc<Region>, irq_sender: IrqVecSender) -> TestDeviceInput {
        TestDeviceInput {
            memory: memory.clone(),
            irq_sender,
        }
    }
}

impl QueueClient for TestDeviceInput {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        let count = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE == 0 && desc.len >= 4 {
                    Some(desc)
                } else {
                    None
                }
            })
            .map(|desc| {
                U32Access::write(self.memory.deref(), desc.addr, 0xa5a55a5a);
                4
            })
            .fold(0, |acc, c| { acc + c });
        if count == 0 {
            return Err(super::queue::Error::ClientError("no valid buffer!".to_string()));
        }
        queue.set_used(desc_head, count as u32)?;
        self.irq_sender.send().unwrap();
        Ok(true)
    }
}

struct TestDeviceOutput {
    memory: Arc<Region>,
}

impl TestDeviceOutput {
    fn new(memory: &Arc<Region>) -> TestDeviceOutput {
        TestDeviceOutput {
            memory: memory.clone(),
        }
    }
}

impl QueueClient for TestDeviceOutput {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        let count = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE == 1 && desc.len == 4 {
                    Some(desc)
                } else {
                    None
                }
            })
            .map(|desc| {
                println!("get data {:#x}", U32Access::read(self.memory.deref(), desc.addr));
                4
            })
            .fold(0, |acc, c| { acc + c });
        if count == 0 {
            return Err(super::queue::Error::ClientError("no valid buffer!".to_string()));
        }
        queue.set_used(desc_head, count as u32)?;
        Ok(true)
    }
}

