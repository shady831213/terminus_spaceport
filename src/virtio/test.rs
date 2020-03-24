use super::queue::{Queue, QueueSetting, DescMeta, RingUsedMetaElem, QueueClient, DESC_F_WRITE, DefaultQueueServer, QueueServer, RingAvailMetaElem, RingMetaHeader, RingMeta};
use std::sync::Arc;
use std::mem;
use crate::memory::{Region, BytesAccess, GHEAP, Heap, U32Access};
use std::ops::Deref;
use super::irq::IrqSignal;
use std::cell::RefCell;

struct TestQueueClient<'a> {
    memory: Arc<Region>,
    irq: Arc<IrqSignal<'a>>,
}

impl<'a> TestQueueClient<'a> {
    fn new(memory: &Arc<Region>, irq: &Arc<IrqSignal<'a>>) -> TestQueueClient<'a> {
        TestQueueClient {
            memory: Arc::clone(memory),
            irq: Arc::clone(irq),
        }
    }
}

impl<'a> QueueClient for TestQueueClient<'a> {
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
        self.irq.send_irq(0).unwrap();
        Ok(true)
    }
}

struct TestQueueServer(DefaultQueueServer);

impl TestQueueServer {
    fn irq_handler(&self, memory: &Region) {
        let (used, desc_iter) = self.pop_used().unwrap();
        for desc_res in desc_iter {
            let (idx, desc) = desc_res.unwrap();
            if desc.flags & DESC_F_WRITE == 0 {
                assert_eq!(U32Access::read(memory, desc.addr), !(0xdeadbeaf as u32))
            } else {
                assert_eq!(U32Access::read(memory, desc.addr), 0xdeadbeaf)
            }
            println!("desc:{}, data:{:#x}", idx, U32Access::read(memory, desc.addr))
        }
        self.free_used(&used).unwrap();
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
    let irq = Arc::new(IrqSignal::new(1));
    irq.set_enable(0).unwrap();
    let client = TestQueueClient::new(&memory, &irq);

    let queue = Arc::new({
        let mut q = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16, manual_recv: false });
        q.bind_client(client);
        q
    });


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


    let server = Arc::new(TestQueueServer(DefaultQueueServer::new(&queue)));
    server.init().unwrap();

    let irq_cnt = Arc::new(RefCell::new(0));

    irq.bind_handler(0, {
        let server_handle = Arc::clone(&server);
        let mut _cnt = Arc::clone(&irq_cnt);
        move || {
            server_handle.irq_handler(memory.deref());
            *_cnt.deref().borrow_mut() += 1;
        }
    }).unwrap();


    let read_mem = heap.alloc(4, 4).unwrap();
    let write_mem = heap.alloc(4, 4).unwrap();
    U32Access::write(write_mem.deref(), write_mem.info.base, 0xdeadbeaf);
    server.add_to_queue(vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.add_to_queue(vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.add_to_queue(vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.add_to_queue(vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    assert_eq!(*irq_cnt.deref().borrow(), 4);
}