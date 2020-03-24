use super::queue::{Queue, QueueSetting, DescMeta, RingUsedMetaElem, QueueClient, DESC_F_WRITE, DefaultQueueServer, QueueServer, RingAvailMetaElem, RingMetaHeader, RingMeta};
use std::sync::{Arc, Mutex};
use std::mem;
use crate::memory::{Region, BytesAccess, GHEAP, Heap, U32Access};
use std::ops::{Deref, DerefMut};
use super::irq::IrqSignal;
use std::marker::PhantomData;
use std::borrow::BorrowMut;
use std::cell::RefCell;

struct TestQueueClient<'a> {
    memory: Arc<Region>,
    irq: RefCell<Box<dyn FnMut() + 'a>>,
}

impl<'a> TestQueueClient<'a> {
    fn new<F: FnMut() + 'a>(memory: &Arc<Region>, irq: F) -> TestQueueClient<'a> {
        TestQueueClient {
            memory: Arc::clone(memory),
            irq: RefCell::new(Box::new(irq)),
        }
    }
}

impl<'a> QueueClient for TestQueueClient<'a> {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        let writes = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE == 0 {
                    let mut data = vec![0 as u8; desc.len as usize];
                    BytesAccess::read(self.memory.deref(), desc.addr, &mut data);
                    Some(data)
                } else {
                    None
                }
            }).collect::<Vec<_>>();

        let count = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE != 0 {
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
        queue.set_used(desc_head, count as u32);
        (&mut *self.irq.borrow_mut())();
        Ok(true)
    }
}

// struct TestQueueServer {
//     server: DefaultQueueServer
// }
//
// impl TestQueueServer {
//     fn new(queue: &Arc<Queue>) -> TestQueueServer {
//         TestQueueServer {
//             server: DefaultQueueServer::new(queue),
//         }
//     }
//
//     fn get_handler<'a>(&'a self) -> ServerHandler<'a> {
//         ServerHandler {
//             server: self,
//         }
//     }
// }
//
// impl Deref for TestQueueServer {
//     type Target = DefaultQueueServer;
//     fn deref(&self) -> &Self::Target {
//         &self.server
//     }
// }
//
// impl DerefMut for TestQueueServer {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.server
//     }
// }
//
// struct ServerHandler<'a> {
//     server: &'a TestQueueServer,
// }
//
// impl<'a, 'b> IrqHandler<'b> for ServerHandler<'a> {
//     fn handle(&mut self) -> super::irq::Result<()> {
//         let result = self.server.pop_used();
//         println!("{:?}", result);
//         Ok(())
//     }
// }

#[test]
fn queue_basic_test() {
    const QUEUE_SIZE: usize = 10;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let mut queue =Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16, manual_recv: false });
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


    let server = Mutex::new(DefaultQueueServer::new(&queue));
    server.lock().unwrap().init();
    let irq = IrqSignal::new(1);

    let client = TestQueueClient::new(&memory, || {
        irq.send_irq(0);
    });

    irq.bind_handler(0, || {
        let result = server.lock().unwrap().pop_used();
        println!("{:?}", result);
    });

    // queue.bind_client(client);

    let read_mem = heap.alloc(4, 4).unwrap();
    let write_mem = heap.alloc(4, 4).unwrap();
    U32Access::write(read_mem.deref(), read_mem.info.base, 0xdeadbeaf);
    server.lock().unwrap().add_to_queue(vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
}