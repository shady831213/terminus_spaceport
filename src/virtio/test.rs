#![allow(dead_code)]
use super::queue::{Queue, QueueSetting, RingUsedMetaElem, QueueClient, DefaultQueueServer, QueueServer};
use crate::memory::region::{Region, BytesAccess, GHEAP, Heap, U32Access};
use std::ops::Deref;
use crate::irq::{IrqVec, IrqVecSender};
use super::device::Device;
use std::cell::RefCell;
use std::rc::Rc;
use crate::virtio::DESC_F_WRITE;

struct TestDevice {
    virtio_device: Device,
    config: TestDeviceConfig,
}

impl TestDevice {
    pub fn new(memory: &Rc<Region>, irq_sender: IrqVecSender) -> TestDevice {
        let mut virtio_device = Device::new(memory,
                                            irq_sender,
                                            2,
                                            0, 0, 0,
        );
        virtio_device.get_irq_vec().set_enable(0, true).unwrap();
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

        virtio_device.get_irq_vec().set_enable(1, true).unwrap();
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
    memory: Rc<Region>,
    irq_sender: IrqVecSender,
    loop_cnt: RefCell<usize>,
}

impl TestDeviceInput {
    fn new(memory: &Rc<Region>, irq_sender: IrqVecSender) -> TestDeviceInput {
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
                // println!("input:{}, {:#x?}",desc_idx, desc);
                assert_eq!(desc_idx, 0);
                if desc.flags & DESC_F_WRITE == 0 && desc.len >= 4 {
                    Some(desc)
                } else {
                    None
                }
            })
            .map(|desc| {
                U32Access::write(self.memory.deref(), &desc.addr, 0xa5a55a5a);
                4
            })
            .fold(0, |acc, c| { acc + c });
        if count == 0 {
            return Err(super::queue::Error::ClientError("no valid buffer!".to_string()));
        }
        queue.set_used(desc_head, count as u32)?;
        queue.update_last_avail();
        self.irq_sender.send().unwrap();
        *self.loop_cnt.borrow_mut() += 1;
        Ok(true)
    }
}

struct TestDeviceOutput {
    memory: Rc<Region>,
}

impl TestDeviceOutput {
    fn new(memory: &Rc<Region>) -> TestDeviceOutput {
        TestDeviceOutput {
            memory: memory.clone(),
        }
    }
}

impl QueueClient for TestDeviceOutput {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        let count = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (desc_idx, desc) = desc_res.unwrap();
                // println!("output:{}, {:#x?}",desc_idx, desc);
                assert_eq!(desc_idx, 0);
                if desc.flags & DESC_F_WRITE != 0 && desc.len == 4 {
                    Some(desc)
                } else {
                    None
                }
            })
            .map(|desc| {
                assert_eq!(U32Access::read(self.memory.deref(), &desc.addr), 0xdeadbeaf);
                println!("get data {:#x}", U32Access::read(self.memory.deref(), &desc.addr));
                4
            })
            .fold(0, |acc, c| { acc + c });
        if count == 0 {
            return Err(super::queue::Error::ClientError("no valid buffer!".to_string()));
        }
        queue.set_used(desc_head, count as u32)?;
        queue.update_last_avail();
        Ok(true)
    }
}


struct TestDeviceDriver {
    irq_vec: IrqVec,
    input_head: u16,
    input_buffer: Rc<Region>,
    heap: Rc<Heap>,
    input_server: DefaultQueueServer,
    output_server: DefaultQueueServer,
    device: TestDevice,
}

impl TestDeviceDriver {
    fn new(heap: &Rc<Heap>) -> TestDeviceDriver {
        let irq_vec = IrqVec::new(1);
        irq_vec.set_enable(0, true).unwrap();
        let device = TestDevice::new(heap.get_region(), irq_vec.sender(0).unwrap());
        let input_queue = device.virtio_device.get_queue(0);
        let input_buffer = heap.alloc(4, 4).unwrap();
        let mut input_server = DefaultQueueServer::new(&heap);
        input_server.init_queue(input_queue).unwrap();
        irq_vec.binder().bind(0, {
            move || {}
        }).unwrap();
        let input_head = input_server.add_to_queue(&input_queue, &vec![input_buffer.deref()].as_slice(), &vec![].as_slice()).unwrap();
        input_server.notify_queue(&input_queue, input_head).unwrap();

        let output_queue = device.virtio_device.get_queue(1);
        let mut output_server = DefaultQueueServer::new(&heap);
        output_server.init_queue(output_queue).unwrap();
        TestDeviceDriver {
            irq_vec,
            input_head,
            input_buffer,
            heap: heap.clone(),
            input_server,
            output_server,
            device,
        }
    }

    fn read(&self) -> super::queue::Result<Vec<u8>> {
        if self.irq_vec.pending(0).unwrap() && self.device.virtio_device.get_irq_vec().pending(0).unwrap() {
            let queue = self.device.virtio_device.get_queue(0);
            let used = self.input_server.pop_used(queue).unwrap();
            let mut output = vec![];
            queue.desc_iter(used.id as u16).for_each(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                let mut desc_buf: Vec<u8> = vec![0; desc.len as usize];
                BytesAccess::read(self.heap.get_region().deref(), &desc.addr, &mut desc_buf).unwrap();
                output.append(&mut desc_buf);
            });
            assert_eq!(used, RingUsedMetaElem { id: 0, len: 4 });
            self.input_server.free_used(queue, &used, true)?;
            self.irq_vec.set_pending(0, false).unwrap();
            self.device.virtio_device.get_irq_vec().set_pending(0, false).unwrap();
            self.input_server.notify_queue(&queue, self.input_head)?;
            return Ok(output);
        }
        Ok(vec![])
    }

    fn write(&self, input: &[u8]) -> super::queue::Result<usize> {
        let queue = self.device.virtio_device.get_queue(1);
        if let Some(used) = self.output_server.pop_used(queue) {
            self.output_server.free_used(queue, &used, false)?;
        }
        let output_buffer = self.heap.alloc(input.len() as u64, 4).unwrap();
        BytesAccess::write(output_buffer.deref(), &output_buffer.info.base, input).unwrap();
        let head = self.output_server.add_to_queue(queue, &vec![].as_slice(), &vec![output_buffer.deref()].as_slice())?;
        self.output_server.notify_queue(queue, head)?;
        Ok(input.len())
    }
}

#[test]
fn simple_device_test() {
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let heap = Heap::new(&memory);
    let device = TestDeviceDriver::new(&heap);
    device.write(&(0xdeadbeaf as u32).to_le_bytes()).unwrap();
    for _ in 0..4 {
        let mut data_array = [0 as u8; 4];
        data_array.copy_from_slice(device.read().unwrap().as_slice());
        assert_eq!(u32::from_le_bytes(data_array), 0xa5a55a5a);
    }
    assert_eq!(device.read().unwrap(), vec![]);
    device.write(&(0xdeadbeaf as u32).to_le_bytes()).unwrap();
    device.write(&(0xdeadbeaf as u32).to_le_bytes()).unwrap();
    // for info in heap.allocator.lock().unwrap().alloced_blocks.iter() {
    //     println!("{:#x?}", info.car());
    // }
}