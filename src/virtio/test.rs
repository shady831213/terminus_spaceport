use super::queue::{Queue, QueueSetting, DescMeta, RingUsedMetaElem, QueueClient, DESC_F_WRITE, DefaultQueueServer, QueueServer, RingAvailMetaElem, RingMetaHeader, RingMeta};
use std::sync::Arc;
use std::mem;
use crate::memory::{Region, BytesAccess, GHEAP, Heap, U32Access};
use std::ops::Deref;
use crate::irq::{IrqVec, IrqVecSender};
use super::device::Device;
use std::cell::RefCell;
use std::cmp::min;

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
        self.free_used(queue, &used, false).unwrap();
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
    let head = server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.notify_queue(&queue, head).unwrap();
    let head = server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.notify_queue(&queue, head).unwrap();
    let head = server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.notify_queue(&queue, head).unwrap();
    let head = server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.notify_queue(&queue, head).unwrap();
    assert_eq!(*irq_cnt.deref().borrow(), 4);
}

struct TestDevice {
    virtio_device: Device,
    config: TestDeviceConfig,
}

impl TestDevice {
    pub fn new(memory: &Arc<Region>, irq_sender: IrqVecSender) -> TestDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            2,
                                            0, 0, 0,
        );
        virtio_device.get_irq_vec().set_enable(0).unwrap();
        let input_queue = {
            let input = TestDeviceInput::new(memory, virtio_device.get_irq_vec().sender(0).unwrap());
            Queue::new(&memory, QueueSetting { max_queue_size: 1 }, input)
        };
        let output_queue = {
            let output = TestDeviceOutput::new(memory);
            Queue::new(&memory, QueueSetting { max_queue_size: 1 }, output)
        };
        virtio_device.add_queue(input_queue);
        virtio_device.add_queue(output_queue);

        virtio_device.get_irq_vec().set_enable(1).unwrap();
        let config = TestDeviceConfig {
            config1: 0,
            config2: 0,
            irq_sender: virtio_device.get_irq_vec().sender(1).unwrap(),
        };
        TestDevice {
            virtio_device,
            config,
        }
    }
}

struct TestDeviceConfig {
    config1: u64,
    config2: u64,
    irq_sender: IrqVecSender,
}

struct TestDeviceInput {
    memory: Arc<Region>,
    irq_sender: IrqVecSender,
    loop_cnt: RefCell<usize>,
}

impl TestDeviceInput {
    fn new(memory: &Arc<Region>, irq_sender: IrqVecSender) -> TestDeviceInput {
        TestDeviceInput {
            memory: memory.clone(),
            irq_sender,
            loop_cnt: RefCell::new(0),
        }
    }
}

impl QueueClient for TestDeviceInput {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        if *self.loop_cnt.borrow() > 3 {
            println!("input 4 times done!");
            return Ok(false);
        }
        let count = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (desc_idx, desc) = desc_res.unwrap();
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
        *self.loop_cnt.borrow_mut() += 1;
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


struct TestDeviceDriver {
    memory: Arc<Region>,
    irq_vec: IrqVec,
    input_head: u16,
    output_head: u16,
    input_server: DefaultQueueServer,
    output_server: DefaultQueueServer,
    device: TestDevice,
}

impl TestDeviceDriver {
    fn new(memory: &Arc<Region>,
           region: &Arc<Region>) -> TestDeviceDriver {
        let irq_vec = IrqVec::new(1);
        irq_vec.set_enable(0).unwrap();
        let device = TestDevice::new(memory, irq_vec.sender(0).unwrap());
        let heap = Heap::new(region);
        let input_queue = device.virtio_device.get_queue(0);
        let input_buffer = heap.alloc(4, 4).unwrap();
        let input_desc_mem = heap.alloc(input_queue.desc_table_size() as u64, 8).unwrap();
        let input_avail_mem = heap.alloc(input_queue.avail_ring_size() as u64, 2).unwrap();
        let input_used_mem = heap.alloc(input_queue.used_ring_size() as u64, 4).unwrap();
        input_queue.set_desc_addr(input_desc_mem.info.base);
        input_queue.set_avail_addr(input_avail_mem.info.base);
        input_queue.set_used_addr(input_used_mem.info.base);
        let input_server = DefaultQueueServer::new();
        input_server.init_queue(input_queue.deref()).unwrap();

        let input_head = input_server.add_to_queue(&input_queue, &vec![input_buffer.deref()].as_slice(), &vec![].as_slice()).unwrap();
        irq_vec.binder().bind(0, {
            move |irq_status| {
                irq_status.set_pending(0).unwrap();
            }
        }).unwrap();
        input_server.notify_queue(&input_queue, input_head).unwrap();

        let output_queue = device.virtio_device.get_queue(1);
        let output_buffer = heap.alloc(4, 4).unwrap();
        let output_desc_mem = heap.alloc(output_queue.desc_table_size() as u64, 8).unwrap();
        let output_avail_mem = heap.alloc(output_queue.avail_ring_size() as u64, 2).unwrap();
        let output_used_mem = heap.alloc(output_queue.used_ring_size() as u64, 4).unwrap();
        output_queue.set_desc_addr(output_desc_mem.info.base);
        output_queue.set_avail_addr(output_avail_mem.info.base);
        output_queue.set_used_addr(output_used_mem.info.base);
        let output_server = DefaultQueueServer::new();
        output_server.init_queue(output_queue.deref()).unwrap();
        let output_head = output_server.add_to_queue(&output_queue, &vec![].as_slice(), &vec![output_buffer.deref()].as_slice()).unwrap();

        TestDeviceDriver {
            memory: memory.clone(),
            irq_vec,
            input_head,
            output_head,
            input_server,
            output_server,
            device,
        }
    }

    fn read(&self) -> super::queue::Result<Vec<u8>> {
        if self.irq_vec.pending(0).unwrap() && self.device.virtio_device.get_irq_vec().pending(0).unwrap() {
            let input_queue = self.device.virtio_device.get_queue(0);
            let used = self.input_server.pop_used(input_queue.deref()).unwrap();
            let mut output = vec![];
            input_queue.desc_iter(used.id as u16).for_each(|desc_res| {
                let (idx, desc) = desc_res.unwrap();
                let mut desc_buf: Vec<u8> = vec![0; desc.len as usize];
                BytesAccess::read(self.memory.deref(), desc.addr, &mut desc_buf);
                output.append(&mut desc_buf);
            });
            assert_eq!(used, RingUsedMetaElem { id: 0, len: 4 });
            self.input_server.free_used(input_queue.deref(),&used, true)?;
            self.irq_vec.clr_pending(0).unwrap();
            self.device.virtio_device.get_irq_vec().clr_pending(0).unwrap();
            self.input_server.notify_queue(&input_queue, self.input_head)?;
            return Ok(output);
        }
        Ok(vec![])
    }
}

#[test]
fn simple_device_test() {
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let heap = Heap::new(&memory);
    let device_region = heap.alloc(512, 8).unwrap();
    let device = TestDeviceDriver::new(&memory, &device_region);
    for _ in 0..4 {
        let mut data_array = [0 as u8;4];
        data_array.copy_from_slice(device.read().unwrap().as_slice());
        assert_eq!(u32::from_le_bytes(data_array), 0xa5a55a5a);
    }
    assert_eq!(device.read().unwrap(), vec![]);
}