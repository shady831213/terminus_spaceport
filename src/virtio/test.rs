use super::queue::{Queue, QueueSetting, DescMeta, RingUsedMetaElem, QueueClient, DESC_F_WRITE, DefaultQueueServer,QueueServer, RingAvailMetaElem, RingMetaHeader,RingMeta};
use std::mem;
use std::sync::Arc;
use crate::memory::{Region, BytesAccess, GHEAP, Heap};
use std::ops::Deref;
use super::irq::{IrqSignal,IrqHandler};

struct TestQueueClient<'a> {
    memory: Arc<Region>,
    irq_signal:&'a IrqSignal
}

impl<'a> QueueClient for TestQueueClient<'a> {
    fn receive(&self, queue: &Queue, desc_head: u16) -> super::queue::Result<bool> {
        let writes = queue.desc_iter(desc_head)
            .filter_map(|desc_res| {
                let (_, desc) = desc_res.unwrap();
                if desc.flags & DESC_F_WRITE != 0 {
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
        queue.set_used(desc_head, count as u32);
        self.irq_signal.send_irq(0);
        Ok(true)
    }
}

struct TestQueueServer<'a> {
    server:DefaultQueueServer<'a>
}

impl<'a> Deref for TestQueueServer<'a> {
    type Target = DefaultQueueServer<'a>;
    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

impl<'a> IrqHandler for TestQueueServer<'a> {
    fn handle(&mut self) -> super::irq::Result<()> {
        let result = self.server.pop_used();
        // println!("{:?}", result);
        Ok(())
    }
}

#[test]
fn queue_basic_test() {
    const QUEUE_SIZE: usize = 10;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let mut queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16, manual_recv: false });
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

    // let server = TestQueueServer{server:DefaultQueueServer::new}
}