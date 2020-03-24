use crate::memory::{Region, GHEAP, Heap, SizedAccess, U16Access};
use std::sync::Arc;
use std::{mem, result, slice};
use std::ops::Deref;
use std::cmp::min;
use std::num::Wrapping;
use std::marker::{PhantomData, Sized};

const DESC_F_NEXT: u16 = 0x1;
const DESC_F_WRITE: u16 = 0x2;

#[derive(Debug)]
pub enum Error {
    InvalidDesc(String),
    InvalidAvail(String),
    InvalidUsed(String),
    InvalidInit(String),
    ServerError(String),
    MemError(String),
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::MemError(error)
    }
}

type Result<T> = result::Result<T, Error>;

#[derive(Copy, Clone)]
pub struct QueueSetting {
    max_queue_size: u16,
    manual_recv: bool,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct DescMeta {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

impl DescMeta {
    fn empty() -> DescMeta {
        DescMeta {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

struct RingMetaHeader {
    flags: u16,
    idx: u16,
}

impl RingMetaHeader {
    fn empty() -> RingMetaHeader {
        RingMetaHeader {
            flags: 0,
            idx: 0,
        }
    }
}

type RingAvailMetaElem = u16;

#[derive(Copy, Clone)]
pub struct RingUsedMetaElem {
    id: u32,
    len: u32,
}

impl RingUsedMetaElem {
    fn empty() -> RingUsedMetaElem {
        RingUsedMetaElem {
            id: 0,
            len: 0,
        }
    }
}

pub struct Queue {
    setting: QueueSetting,
    memory: Arc<Region>,
    ready: bool,
    queue_size: u16,
    last_avail_idx: u16,
    desc_addr: u64,
    avail_addr: u64,
    used_addr: u64,
}

impl Queue {
    pub fn new(memory: &Arc<Region>, setting: QueueSetting) -> Queue {
        // assert!(max_queue_size.is_power_of_two());
        let max_queue_size = setting.max_queue_size;
        Queue {
            setting,
            memory: Arc::clone(memory),
            ready: false,
            queue_size: max_queue_size,
            last_avail_idx: 0,
            desc_addr: 0,
            avail_addr: 0,
            used_addr: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ready = false;
        self.last_avail_idx = 0;
        self.queue_size = self.setting.max_queue_size;
        self.desc_addr = 0;
        self.avail_addr = 0;
        self.used_addr = 0;
    }

    fn get_queue_size(&self) -> usize {
        min(self.queue_size, self.setting.max_queue_size) as usize
    }

    fn check_idx(&self, idx: u16) -> Result<()> {
        if idx as usize >= self.get_queue_size() {
            Err(Error::InvalidDesc(format!("invalid desc idx! {}", idx)))
        } else {
            Ok(())
        }
    }

    fn desc_addr(&self, idx: u16) -> Result<u64> {
        self.check_idx(idx)?;
        Ok(self.desc_addr + (idx as usize * mem::size_of::<DescMeta>()) as u64)
    }

    fn desc_table_size(&self) -> usize {
        mem::size_of::<DescMeta>() * self.get_queue_size()
    }

    fn avail_ring_size(&self) -> usize {
        mem::size_of::<RingAvailMetaElem>() * self.get_queue_size()
    }

    fn used_ring_size(&self) -> usize {
        mem::size_of::<RingUsedMetaElem>() * self.get_queue_size()
    }

    fn check_range(&self, base: u64, size: u64) -> bool {
        !(base < self.memory.info.base ||
            base >= self.memory.info.base + self.memory.info.size ||
            base + size < self.memory.info.base ||
            base + size >= self.memory.info.base + self.memory.info.size)
    }

    fn avail_elem_addr(&self, idx: u16) -> u64 {
        self.avail_addr + mem::size_of::<RingMetaHeader>() as u64 + (idx as usize % self.get_queue_size()) as u64 * mem::size_of::<RingAvailMetaElem>() as u64
    }

    pub fn set_desc(&self, idx: u16, desc: &DescMeta) -> Result<()> {
        Ok(SizedAccess::write(self.memory.deref(), self.desc_addr(idx)?, desc))
    }

    pub fn get_desc(&self, idx: u16) -> Result<DescMeta> {
        let mut desc = DescMeta::empty();
        SizedAccess::read(self.memory.deref(), self.desc_addr(idx)?, &mut desc);
        Ok(desc)
    }

    pub fn get_avail_idx(&self) -> Wrapping<u16> {
        Wrapping(U16Access::read(self.memory.deref(), self.avail_addr + 2))
    }

    pub fn set_avail_idx(&self, idx: u16) {
        U16Access::write(self.memory.deref(), self.avail_addr + 2, idx)
    }

    pub fn set_avail_desc(&self, avail_idx: u16, desc_idx: u16) -> Result<()> {
        self.check_idx(desc_idx)?;
        U16Access::write(self.memory.deref(), self.avail_elem_addr(avail_idx), desc_idx);
        Ok(())
    }

    pub fn get_used_idx(&self) -> Wrapping<u16> {
        Wrapping(U16Access::read(self.memory.deref(), self.used_addr + 2))
    }

    fn set_used_idx(&self, idx: u16) {
        U16Access::write(self.memory.deref(), self.used_addr + 2, idx)
    }

    fn used_elem_addr(&self, idx: u16) -> u64 {
        self.used_addr + mem::size_of::<RingMetaHeader>() as u64 + (idx as usize % self.get_queue_size()) as u64 * mem::size_of::<RingUsedMetaElem>() as u64
    }

    pub fn get_used_elem(&self, used_idx: u16) -> RingUsedMetaElem {
        let mut elem = RingUsedMetaElem::empty();
        SizedAccess::read(self.memory.deref(), self.used_elem_addr(used_idx), &mut elem);
        elem
    }

    fn set_used_elem(&self, used_idx: u16, elem: &RingUsedMetaElem) -> Result<()> {
        self.check_idx(elem.id as u16)?;
        SizedAccess::write(self.memory.deref(), self.used_elem_addr(used_idx), elem);
        Ok(())
    }

    pub fn set_used(&self, desc_idx: u16, len: u32) -> Result<()> {
        let mut used_idx = self.get_used_idx();
        let used_elem = RingUsedMetaElem {
            id: desc_idx as u32,
            len,
        };
        self.set_used_elem(used_idx.0, &used_elem)?;
        used_idx += Wrapping(1);
        self.set_used_idx(used_idx.0);
        Ok(())
    }

    pub fn check_init(&self) -> Result<()> {
        if self.ready {
            return Err(Error::InvalidInit("init when ready is not unset!".to_string()));
        }
        if !self.check_range(self.desc_addr, self.desc_table_size() as u64) {
            return Err(Error::InvalidInit(format!("invalid desc addr {:#016x}", self.desc_addr)));
        }
        if !self.check_range(self.avail_addr, (self.avail_ring_size() + mem::size_of::<RingMetaHeader>()) as u64) {
            return Err(Error::InvalidInit(format!("invalid avail addr {:#016x}", self.avail_addr)));
        }
        if !self.check_range(self.used_addr, (self.used_ring_size() + mem::size_of::<RingMetaHeader>()) as u64) {
            return Err(Error::InvalidInit(format!("invalid used addr {:#016x}", self.used_addr)));
        }
        Ok(())
    }

    pub fn desc_iter<'a>(&'a self, idx: u16) -> DescIter<'a> {
        DescIter::new(self, idx, PhantomData)
    }

    pub fn avail_iter<'a>(&'a self) -> AvailIter<'a> {
        let mut header = RingMetaHeader { flags: 0, idx: 0 };
        SizedAccess::read(self.memory.deref(), self.avail_addr, &mut header);
        AvailIter::new(self,
                       Wrapping(header.idx),
                       Wrapping(self.last_avail_idx),
                       PhantomData)
    }
}

pub struct AvailIter<'a> {
    queue: &'a Queue,
    end_idx: Wrapping<u16>,
    next_idx: Wrapping<u16>,
    marker: PhantomData<&'a Queue>,
}

impl<'a> AvailIter<'a> {
    fn new(queue: &'a Queue,
           end_idx: Wrapping<u16>,
           next_idx: Wrapping<u16>,
           marker: PhantomData<&'a Queue>) -> AvailIter<'a> {
        AvailIter {
            queue,
            end_idx,
            next_idx,
            marker,
        }
    }
}

impl<'a> Iterator for AvailIter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end_idx == self.next_idx {
            return None;
        }

        let ring_elem_addr = self.queue.avail_elem_addr(self.next_idx.0);
        let mut desc_idx = 0 as RingAvailMetaElem;
        SizedAccess::read(self.queue.memory.deref(), ring_elem_addr, &mut desc_idx);
        self.next_idx += Wrapping(1);
        Some(desc_idx)
    }
}

pub struct DescIter<'a> {
    queue: &'a Queue,
    ttl: u16,
    head: u16,
    idx: u16,
    last: bool,
    marker: PhantomData<&'a Queue>,
}

impl<'a> DescIter<'a> {
    fn new(queue: &'a Queue, head: u16, marker: PhantomData<&'a Queue>) -> DescIter<'a> {
        DescIter {
            queue,
            ttl: 1,
            head,
            idx: head,
            last: false,
            marker,
        }
    }

    fn has_next(&self, meta: &DescMeta) -> bool {
        meta.flags & DESC_F_NEXT == DESC_F_NEXT
    }

    fn dead_loop(&self) -> bool {
        self.ttl as usize >= self.queue.get_queue_size()
    }
}

impl<'a> Iterator for DescIter<'a> {
    type Item = Result<(u16, DescMeta)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.last {
            return None;
        }
        match self.queue.get_desc(self.idx) {
            Ok(meta) => {
                if self.dead_loop() {
                    self.last = true;
                    Some(Err(Error::InvalidDesc(format!("infinity descriptor chain start from {}!", self.head))))
                } else {
                    let cur_idx = self.idx;
                    self.ttl += 1;
                    self.idx = meta.next;
                    self.last = !self.has_next(&meta);
                    Some(Ok((cur_idx, meta)))
                }
            }
            Err(e) => Some(Err(e))
        }
    }
}

pub trait QueueServer {
    fn add_to_queue(&mut self, inputs: &[&Region], outputs: &[&Region]) -> Result<u16>;
    fn has_used(&self) -> bool;
    fn num_available_desc(&self) -> usize;
    fn pop_used(&mut self) -> Result<Option<RingUsedMetaElem>>;
}

pub struct DefaultQueueServer<'a> {
    queue: &'a Queue,
    num_used: u16,
    free_head: u16,
    last_used_idx: u16,
}

impl<'a> DefaultQueueServer<'a> {
    fn new(queue: &'a Queue) -> DefaultQueueServer<'a> {
        DefaultQueueServer {
            queue,
            num_used: 0,
            free_head: 0,
            last_used_idx: 0,
        }
    }
    fn init(&mut self) -> Result<()> {
        self.queue.check_init()?;
        let mut descs = vec![DescMeta::empty(); self.queue.get_queue_size()];
        for i in 0..(descs.len() - 1) {
            descs[i].flags |= DESC_F_NEXT;
            descs[i].next = i as u16 + 1;
        }
        descs.last_mut().unwrap().flags |= DESC_F_NEXT;
        descs.last_mut().unwrap().next = 0;
        for (i, desc) in descs.iter().enumerate() {
            self.queue.set_desc(i as u16, desc)?;
        }
        Ok(())
    }

    fn free_desc(&mut self, idx: u16) -> Result<()> {
        for desc_res in self.queue.desc_iter(idx) {
            let (desc_idx, mut desc) = desc_res?;
            self.num_used -= 1;
            if desc.flags & DESC_F_NEXT == 0 {
                desc.next = self.free_head;
                desc.flags |= DESC_F_NEXT;
                self.queue.set_desc(desc_idx, &desc)?;
            }
        }
        self.free_head = idx;
        Ok(())
    }
}

impl<'a> QueueServer for DefaultQueueServer<'a> {
    fn add_to_queue(&mut self, inputs: &[&Region], outputs: &[&Region]) -> Result<u16> {
        if inputs.is_empty() && outputs.is_empty() {
            return Err(Error::ServerError("inputs and outputs are both empty!".to_string()));
        }
        if inputs.len() + outputs.len() + self.num_used as usize > self.queue.get_queue_size() as usize {
            return Err(Error::ServerError("inputs and outputs are too big!".to_string()));
        }

        let head = self.free_head;
        let mut last = self.free_head;
        let mut desc_iter = self.queue.desc_iter(self.free_head);
        for input in inputs.iter() {
            let (_, mut desc) = desc_iter.next().unwrap()?;
            desc.addr = input.info.base;
            desc.len = input.info.size as u32;
            desc.flags = 0;
            desc.flags |= DESC_F_NEXT;
            self.queue.set_desc(self.free_head, &desc)?;
            last = self.free_head;
            self.free_head = desc.next;
        }
        for output in outputs.iter() {
            let (_, mut desc) = desc_iter.next().unwrap()?;
            desc.addr = output.info.base;
            desc.len = output.info.size as u32;
            desc.flags = 0;
            desc.flags |= DESC_F_NEXT | DESC_F_WRITE;
            self.queue.set_desc(self.free_head, &desc)?;
            last = self.free_head;
            self.free_head = desc.next;
        }
        {
            let desc = &mut self.queue.get_desc(last)?;
            desc.flags = desc.flags & !DESC_F_NEXT;
            self.queue.set_desc(last, desc)?;
        }

        self.num_used += (inputs.len() + outputs.len()) as u16;
        let mut avail_idx = self.queue.get_avail_idx();
        self.queue.set_avail_desc(avail_idx.0, head)?;
        avail_idx += Wrapping(1);
        self.queue.set_avail_idx(avail_idx.0);

        Ok(self.free_head)
    }

    fn has_used(&self) -> bool {
        self.queue.ready && (self.last_used_idx != self.queue.get_used_idx().0)
    }

    fn num_available_desc(&self) -> usize {
        self.queue.get_queue_size() - self.num_used as usize
    }

    fn pop_used(&mut self) -> Result<Option<RingUsedMetaElem>> {
        if !self.has_used() {
            return Ok(None);
        }
        let last_used = self.last_used_idx % self.queue.get_queue_size() as u16;
        let used_elem = self.queue.get_used_elem(last_used);
        self.free_desc(used_elem.id as u16)?;
        self.last_used_idx = self.queue.get_used_idx().0;

        Ok(Some(used_elem))
    }
}

#[cfg(test)]
struct RingMeta<T: ?Sized> {
    info: RingMetaHeader,
    ring: T,
}

#[test]
fn get_desc_test() {
    const QUEUE_SIZE: usize = 2;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let mut queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16, manual_recv: false });
    let heap = Heap::new(&memory);
    let desc_mem = heap.alloc(mem::size_of::<DescMeta>() as u64 * queue.get_queue_size() as u64, 4).unwrap();
    let avail_ring: RingMeta<[RingAvailMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 0,
        },
        ring: [0 as RingAvailMetaElem; QUEUE_SIZE],
    };
    let avail_mem = heap.alloc(mem::size_of_val(&avail_ring) as u64, 2).unwrap();
    let used_ring: RingMeta<[RingUsedMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 0,
        },
        ring: [RingUsedMetaElem::empty(); QUEUE_SIZE],
    };
    let used_mem = heap.alloc(mem::size_of_val(&used_ring) as u64, 2).unwrap();
    queue.desc_addr = desc_mem.info.base;
    queue.avail_addr = avail_mem.info.base;
    queue.used_addr = used_mem.info.base;

    let mut server = DefaultQueueServer::new(&queue);
    server.init().unwrap();
    queue.set_desc(0, &DescMeta {
        addr: 0xa5a5,
        len: 0x5a5a,
        flags: 0,
        next: 0xbeaf,
    }).unwrap();
    let mut desc_iter = queue.desc_iter(0);
    let desc = desc_iter.next().unwrap().unwrap();
    assert_eq!(desc.1, DescMeta {
        addr: 0xa5a5,
        len: 0x5a5a,
        flags: 0,
        next: 0xbeaf,
    });
    assert_eq!(desc_iter.next().is_none(), true);
}

#[test]
fn avail_iter_test() {
    const QUEUE_SIZE: usize = 10;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let mut queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16, manual_recv: false });
    let heap = Heap::new(&memory);
    let desc_mem = heap.alloc(mem::size_of::<DescMeta>() as u64 * queue.get_queue_size() as u64, 4).unwrap();
    let mut avail_ring: RingMeta<[RingAvailMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 14,
        },
        ring: [0 as RingAvailMetaElem; QUEUE_SIZE],
    };
    avail_ring.ring[1] = 3;
    avail_ring.ring[2] = 5;
    avail_ring.ring[3] = 8;
    avail_ring.ring[4] = 7;
    avail_ring.ring[5] = 1;
    let avail_mem = heap.alloc(mem::size_of_val(&avail_ring) as u64, 2).unwrap();
    let used_ring: RingMeta<[RingUsedMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 0,
        },
        ring: [RingUsedMetaElem::empty(); QUEUE_SIZE],
    };
    let used_mem = heap.alloc(mem::size_of_val(&used_ring) as u64, 2).unwrap();
    queue.desc_addr = desc_mem.info.base;
    queue.avail_addr = avail_mem.info.base;
    queue.used_addr = used_mem.info.base;


    assert_eq!(avail_mem.info.size, 4 + 10 * 2);
    SizedAccess::write(avail_mem.deref(), avail_mem.info.base, &avail_ring);
    queue.last_avail_idx = 11;
    for pair in queue.avail_iter().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + queue.last_avail_idx as usize % queue.get_queue_size()], pair.1)
    }
    queue.last_avail_idx = 14;
    U16Access::write(avail_mem.deref(), avail_mem.info.base + 2, 16);
    for pair in queue.avail_iter().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + queue.last_avail_idx as usize % queue.get_queue_size()], pair.1)
    }
}

#[test]
fn add_to_queue_test() {
    const QUEUE_SIZE: usize = 10;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let mut queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16, manual_recv: false });
    let heap = Heap::new(&memory);
    let desc_mem = heap.alloc(mem::size_of::<DescMeta>() as u64 * queue.get_queue_size() as u64, 4).unwrap();
    let avail_ring: RingMeta<[RingAvailMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 0,
        },
        ring: [0 as RingAvailMetaElem; QUEUE_SIZE],
    };
    let avail_mem = heap.alloc(mem::size_of_val(&avail_ring) as u64, 2).unwrap();
    let used_ring: RingMeta<[RingUsedMetaElem; QUEUE_SIZE]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 0,
        },
        ring: [RingUsedMetaElem::empty(); QUEUE_SIZE],
    };
    let used_mem = heap.alloc(mem::size_of_val(&used_ring) as u64, 2).unwrap();
    queue.desc_addr = desc_mem.info.base;
    queue.avail_addr = avail_mem.info.base;
    queue.used_addr = used_mem.info.base;

    let mut server = DefaultQueueServer::new(&queue);
    server.init().unwrap();
    for i in 0..(queue.get_queue_size() - 1) {
        let desc = queue.get_desc(i as u16).unwrap();
        assert_eq!(desc.next, i as u16 + 1)
    }

    let read_mem = heap.alloc(7, 1).unwrap();
    let write_mem = heap.alloc(6, 1).unwrap();

    server.add_to_queue(vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();

    let read_mem1 = heap.alloc(7, 1).unwrap();
    let write_mem1 = heap.alloc(6, 1).unwrap();
    let write_mem2 = heap.alloc(6, 1).unwrap();
    let write_mem3 = heap.alloc(6, 1).unwrap();

    server.add_to_queue(vec![read_mem1.deref()].as_slice(), vec![write_mem1.deref(), write_mem2.deref(), write_mem3.deref()].as_slice()).unwrap();

    let mut avail_iter = queue.avail_iter();
    let mut desc_iter = queue.desc_iter(avail_iter.next().unwrap());
    assert_eq!(desc_iter.next().unwrap().unwrap().1, DescMeta {
        addr: read_mem.info.base,
        len: read_mem.info.size as u32,
        flags: DESC_F_NEXT,
        next: 1,
    });
    assert_eq!(desc_iter.next().unwrap().unwrap().1, DescMeta {
        addr: write_mem.info.base,
        len: write_mem.info.size as u32,
        flags: DESC_F_WRITE,
        next: 2,
    });

    assert!(desc_iter.next().is_none());

    let mut desc_iter = queue.desc_iter(avail_iter.next().unwrap());
    assert_eq!(desc_iter.next().unwrap().unwrap().1, DescMeta {
        addr: read_mem1.info.base,
        len: read_mem1.info.size as u32,
        flags: DESC_F_NEXT,
        next: 3,
    });

    for (i, rest) in desc_iter.enumerate() {
        let (desc_idx, desc) = rest.unwrap();
        assert_eq!(desc.len, 6);
        assert_eq!(desc.flags & DESC_F_WRITE, DESC_F_WRITE);
        assert_eq!(desc.next, i as u16 + 4);
        assert_eq!(desc_idx, i as u16 + 3);
        if desc.flags & DESC_F_NEXT == 0 {
            assert_eq!(i, 2);
        }
    }
}