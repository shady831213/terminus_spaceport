use crate::memory::region::{Region, GHEAP, Heap, SizedAccess, U16Access};
use std::{mem, result};
use std::ops::Deref;
use std::cmp::min;
use std::num::Wrapping;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::rc::Rc;

pub const DESC_F_NEXT: u16 = 0x1;
pub const DESC_F_WRITE: u16 = 0x2;

#[derive(Debug)]
pub enum Error {
    InvalidDesc(String),
    InvalidAvail(String),
    InvalidUsed(String),
    InvalidInit(String),
    ServerError(String),
    ClientError(String),
    NotReady,
    MemError(String),
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::MemError(error)
    }
}


pub type Result<T> = result::Result<T, Error>;

#[derive(Copy, Clone)]
pub struct QueueSetting {
    pub max_queue_size: u16,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct DescMeta {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
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


pub struct RingMetaHeader {
    flags: u16,
    idx: u16,
}

impl RingMetaHeader {
    pub fn empty() -> RingMetaHeader {
        RingMetaHeader {
            flags: 0,
            idx: 0,
        }
    }
}

pub type RingAvailMetaElem = u16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RingUsedMetaElem {
    pub id: u32,
    pub len: u32,
}

impl RingUsedMetaElem {
    pub fn empty() -> RingUsedMetaElem {
        RingUsedMetaElem {
            id: 0,
            len: 0,
        }
    }
}

pub struct Queue {
    setting: QueueSetting,
    memory: Rc<Region>,
    client: Box<dyn QueueClient>,
    ready: RefCell<bool>,
    queue_size: u16,
    last_avail_idx: RefCell<Wrapping<u16>>,
    desc_addr: RefCell<u64>,
    avail_addr: RefCell<u64>,
    used_addr: RefCell<u64>,
}

impl Queue {
    pub fn new(memory: &Rc<Region>, setting: QueueSetting, client: impl QueueClient + 'static) -> Queue {
        // assert!(max_queue_size.is_power_of_two());
        let max_queue_size = setting.max_queue_size;
        Queue {
            setting,
            memory: Rc::clone(memory),
            client: Box::new(client),
            ready: RefCell::new(false),
            queue_size: max_queue_size,
            last_avail_idx: RefCell::new(Wrapping(0)),
            desc_addr: RefCell::new(0),
            avail_addr: RefCell::new(0),
            used_addr: RefCell::new(0),
        }
    }

    pub fn reset(&mut self) {
        *self.ready.borrow_mut() = false;
        *self.last_avail_idx.borrow_mut() = Wrapping(0);
        self.queue_size = self.setting.max_queue_size;
        *self.desc_addr.borrow_mut() = 0;
        *self.avail_addr.borrow_mut() = 0;
        *self.used_addr.borrow_mut() = 0;
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
        Ok(self.get_desc_addr() + (idx as usize * mem::size_of::<DescMeta>()) as u64)
    }

    pub fn desc_table_size(&self) -> usize {
        mem::size_of::<DescMeta>() * self.get_queue_size()
    }

    pub fn avail_ring_size(&self) -> usize {
        mem::size_of::<RingAvailMetaElem>() * self.get_queue_size()
    }

    pub fn used_ring_size(&self) -> usize {
        mem::size_of::<RingUsedMetaElem>() * self.get_queue_size()
    }

    fn check_range(&self, base: u64, size: u64) -> bool {
        !(base < self.memory.info.base ||
            base >= self.memory.info.base + self.memory.info.size ||
            base + size < self.memory.info.base ||
            base + size >= self.memory.info.base + self.memory.info.size)
    }

    fn avail_elem_addr(&self, idx: u16) -> u64 {
        self.get_avail_addr() + mem::size_of::<RingMetaHeader>() as u64 + (idx as usize % self.get_queue_size()) as u64 * mem::size_of::<RingAvailMetaElem>() as u64
    }

    pub fn get_ready(&self) -> bool {
        *self.ready.borrow()
    }

    pub fn set_ready(&self, ready: bool) {
        *self.ready.borrow_mut() = ready
    }

    pub fn get_desc_addr(&self) -> u64 {
        *self.desc_addr.borrow()
    }

    pub fn set_desc_addr(&self, desc_addr: u64) {
        *self.desc_addr.borrow_mut() = desc_addr
    }

    pub fn get_avail_addr(&self) -> u64 {
        *self.avail_addr.borrow()
    }

    pub fn set_avail_addr(&self, avail_addr: u64) {
        *self.avail_addr.borrow_mut() = avail_addr
    }

    pub fn get_used_addr(&self) -> u64 {
        *self.used_addr.borrow()
    }

    pub fn set_used_addr(&self, used_addr: u64) {
        *self.used_addr.borrow_mut() = used_addr
    }

    pub fn get_queue_size(&self) -> usize {
        min(self.queue_size, self.setting.max_queue_size) as usize
    }

    pub fn set_desc(&self, idx: u16, desc: &DescMeta) -> Result<()> {
        SizedAccess::write(self.memory.deref(), &self.desc_addr(idx)?, desc);
        Ok(())
    }

    pub fn get_desc(&self, idx: u16) -> Result<DescMeta> {
        let mut desc = DescMeta::empty();
        SizedAccess::read(self.memory.deref(), &self.desc_addr(idx)?, &mut desc);
        Ok(desc)
    }

    pub fn get_avail_idx(&self) -> Result<Wrapping<u16>> {
        Ok(Wrapping(U16Access::read(self.memory.deref(), &(self.get_avail_addr() + 2))))
    }

    pub fn set_avail_idx(&self, idx: u16) -> Result<()> {
        U16Access::write(self.memory.deref(), &(self.get_avail_addr() + 2), idx);
        Ok(())
    }

    pub fn set_avail_desc(&self, avail_idx: u16, desc_idx: u16) -> Result<()> {
        self.check_idx(desc_idx)?;
        U16Access::write(self.memory.deref(), &self.avail_elem_addr(avail_idx), desc_idx);
        Ok(())
    }

    pub fn get_used_idx(&self) -> Result<Wrapping<u16>> {
        Ok(Wrapping(U16Access::read(self.memory.deref(), &(self.get_used_addr() + 2))))
    }

    fn set_used_idx(&self, idx: u16) -> Result<()> {
        U16Access::write(self.memory.deref(), &(self.get_used_addr() + 2), idx);
        Ok(())
    }

    fn used_elem_addr(&self, idx: u16) -> u64 {
        self.get_used_addr() + mem::size_of::<RingMetaHeader>() as u64 + (idx as usize % self.get_queue_size()) as u64 * mem::size_of::<RingUsedMetaElem>() as u64
    }

    pub fn get_used_elem(&self, used_idx: u16) -> Result<RingUsedMetaElem> {
        let mut elem = RingUsedMetaElem::empty();
        SizedAccess::read(self.memory.deref(), &self.used_elem_addr(used_idx), &mut elem);
        Ok(elem)
    }

    fn set_used_elem(&self, used_idx: u16, elem: &RingUsedMetaElem) -> Result<()> {
        self.check_idx(elem.id as u16)?;
        SizedAccess::write(self.memory.deref(), &self.used_elem_addr(used_idx), elem);
        Ok(())
    }

    pub fn set_used(&self, desc_idx: u16, len: u32) -> Result<()> {
        self.check_ready()?;
        let mut used_idx = self.get_used_idx()?;
        let used_elem = RingUsedMetaElem {
            id: desc_idx as u32,
            len,
        };
        self.set_used_elem(used_idx.0, &used_elem)?;
        used_idx += Wrapping(1);
        self.set_used_idx(used_idx.0)?;
        Ok(())
    }

    pub fn check_init(&self) -> Result<()> {
        if self.get_ready() {
            return Err(Error::InvalidInit("init when ready is not unset!".to_string()));
        }
        if !self.check_range(self.get_desc_addr(), self.desc_table_size() as u64) {
            return Err(Error::InvalidInit(format!("invalid desc addr {:#016x}", self.get_desc_addr())));
        }
        if !self.check_range(self.get_avail_addr(), (self.avail_ring_size() + mem::size_of::<RingMetaHeader>()) as u64) {
            return Err(Error::InvalidInit(format!("invalid avail addr {:#016x}", self.get_avail_addr())));
        }
        if !self.check_range(self.get_used_addr(), (self.used_ring_size() + mem::size_of::<RingMetaHeader>()) as u64) {
            return Err(Error::InvalidInit(format!("invalid used addr {:#016x}", self.get_used_addr())));
        }
        Ok(())
    }

    fn check_ready(&self) -> Result<()> {
        if !self.get_ready() {
            Err(Error::NotReady)
        } else {
            Ok(())
        }
    }

    pub fn desc_iter(&self, idx: u16) -> DescIter {
        DescIter::new(self, idx, PhantomData)
    }

    fn avail_iter(&self) -> Result<AvailIter> {
        let mut header = RingMetaHeader { flags: 0, idx: 0 };
        SizedAccess::read(self.memory.deref(), &self.get_avail_addr(), &mut header);
        Ok(AvailIter::new(self,
                          Wrapping(header.idx),
                          *self.last_avail_idx.borrow(),
                          PhantomData))
    }

    pub fn notify_client(&self) -> Result<()> {
        self.check_ready()?;
        for desc_head in self.avail_iter()? {
            if !self.client.receive(self, desc_head)? {
                return Ok(());
            }
            *self.last_avail_idx.borrow_mut() += Wrapping(1);
        }
        Ok(())
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
        SizedAccess::read(self.queue.memory.deref(), &ring_elem_addr, &mut desc_idx);
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
        self.ttl as usize > self.queue.get_queue_size()
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
    fn init_queue(&mut self, queue: &Queue) -> Result<()>;
    fn add_to_queue(&self, queue: &Queue, inputs: &[&Region], outputs: &[&Region]) -> Result<u16>;
    fn notify_queue(&self, queue: &Queue, head: u16) -> Result<()>;
    fn has_used(&self, queue: &Queue) -> bool;
    fn num_available_desc(&self, queue: &Queue) -> usize;
    fn pop_used(&self, queue: &Queue) -> Option<RingUsedMetaElem>;
    fn free_used(&self, queue: &Queue, used: &RingUsedMetaElem, keep_desc: bool) -> Result<()>;
}


pub trait QueueClient {
    fn receive(&self, queue: &Queue, desc_head: u16) -> Result<bool>;
}

pub struct DefaultQueueServer {
    heap: Rc<Heap>,
    desc_region: Option<Rc<Region>>,
    avail_region: Option<Rc<Region>>,
    used_region: Option<Rc<Region>>,
    num_used: RefCell<u16>,
    free_head: RefCell<u16>,
    last_used_idx: RefCell<u16>,
}

impl DefaultQueueServer {
    pub fn new(heap: &Rc<Heap>) -> DefaultQueueServer {
        DefaultQueueServer {
            heap: heap.clone(),
            desc_region: None,
            avail_region: None,
            used_region: None,
            num_used: RefCell::new(0),
            free_head: RefCell::new(0),
            last_used_idx: RefCell::new(0),
        }
    }

    fn free_desc(&self, queue: &Queue, idx: u16) -> Result<()> {
        for desc_res in queue.desc_iter(idx) {
            let (desc_idx, mut desc) = desc_res?;
            *self.num_used.borrow_mut() -= 1;
            if desc.flags & DESC_F_NEXT == 0 {
                desc.next = *self.free_head.borrow();
                desc.flags |= DESC_F_NEXT;
                queue.set_desc(desc_idx, &desc)?;
            }
        }
        *self.free_head.borrow_mut() = idx;
        Ok(())
    }
}

impl QueueServer for DefaultQueueServer {
    fn init_queue(&mut self, queue: &Queue) -> Result<()> {
        let desc_region = self.heap.alloc(queue.desc_table_size() as u64, 8)?;
        let avail_region = self.heap.alloc(queue.avail_ring_size() as u64, 2)?;
        let used_region = self.heap.alloc(queue.used_ring_size() as u64, 4)?;
        queue.set_desc_addr(desc_region.info.base);
        queue.set_avail_addr(avail_region.info.base);
        queue.set_used_addr(used_region.info.base);
        self.desc_region = Some(desc_region);
        self.avail_region = Some(avail_region);
        self.used_region = Some(used_region);
        queue.check_init()?;
        let mut descs = vec![DescMeta::empty(); queue.get_queue_size()];
        for i in 0..(descs.len() - 1) {
            descs[i].flags |= DESC_F_NEXT;
            descs[i].next = i as u16 + 1;
        }
        descs.last_mut().unwrap().flags |= DESC_F_NEXT;
        descs.last_mut().unwrap().next = 0;
        for (i, desc) in descs.iter().enumerate() {
            queue.set_desc(i as u16, desc)?;
        }
        queue.set_ready(true);
        Ok(())
    }

    fn add_to_queue(&self, queue: &Queue, inputs: &[&Region], outputs: &[&Region]) -> Result<u16> {
        if inputs.is_empty() & outputs.is_empty() {
            return Err(Error::ServerError("inputs and outputs are both empty!".to_string()));
        }
        if inputs.len() + outputs.len() + *self.num_used.borrow() as usize > queue.get_queue_size() as usize {
            return Err(Error::ServerError("inputs and outputs are too big!".to_string()));
        }

        let head = *self.free_head.borrow();
        let mut last = *self.free_head.borrow();
        let mut desc_iter = queue.desc_iter(*self.free_head.borrow());
        for input in inputs.iter() {
            let (_, mut desc) = desc_iter.next().unwrap()?;
            desc.addr = input.info.base;
            desc.len = input.info.size as u32;
            desc.flags = 0;
            desc.flags |= DESC_F_NEXT;
            queue.set_desc(*self.free_head.borrow(), &desc)?;
            last = *self.free_head.borrow();
            *self.free_head.borrow_mut() = desc.next;
        }
        for output in outputs.iter() {
            let (_, mut desc) = desc_iter.next().unwrap()?;
            desc.addr = output.info.base;
            desc.len = output.info.size as u32;
            desc.flags = 0;
            desc.flags |= DESC_F_NEXT | DESC_F_WRITE;
            queue.set_desc(*self.free_head.borrow(), &desc)?;
            last = *self.free_head.borrow();
            *self.free_head.borrow_mut() = desc.next;
        }
        {
            let desc = &mut queue.get_desc(last)?;
            desc.flags = desc.flags & !DESC_F_NEXT;
            queue.set_desc(last, desc)?;
        }
        *self.num_used.borrow_mut() += (inputs.len() + outputs.len()) as u16;

        Ok(head)
    }

    fn notify_queue(&self, queue: &Queue, head: u16) -> Result<()> {
        let mut avail_idx = queue.get_avail_idx()?;
        queue.set_avail_desc(avail_idx.0, head)?;
        avail_idx += Wrapping(1);
        queue.set_avail_idx(avail_idx.0)?;

        queue.notify_client()
    }

    fn has_used(&self, queue: &Queue) -> bool {
        queue.get_ready() & (*self.last_used_idx.borrow() != queue.get_used_idx().unwrap().0)
    }

    fn num_available_desc(&self, queue: &Queue) -> usize {
        queue.get_queue_size() - *self.num_used.borrow() as usize
    }

    fn pop_used(&self, queue: &Queue) -> Option<RingUsedMetaElem> {
        if !self.has_used(queue) {
            return None;
        }

        let last_used = *self.last_used_idx.borrow() % queue.get_queue_size() as u16;
        let used_elem = queue.get_used_elem(last_used).unwrap();

        Some(used_elem)
    }

    fn free_used(&self, queue: &Queue, used: &RingUsedMetaElem, keep_desc: bool) -> Result<()> {
        if !self.has_used(queue) {
            return Err(Error::ServerError("there's no used, free_used() shouldn't be called!".to_string()));
        }
        if !keep_desc {
            self.free_desc(queue, used.id as u16)?;
        }
        *self.last_used_idx.borrow_mut() = queue.get_used_idx()?.0;
        Ok(())
    }
}

#[cfg(test)]
struct DummyClient();

#[cfg(test)]
impl QueueClient for DummyClient {
    fn receive(&self, _: &Queue, _: u16) -> Result<bool> {
        Ok(false)
    }
}

#[cfg(test)]
pub struct RingMeta<T: ?Sized> {
    pub info: RingMetaHeader,
    pub ring: T,
}

#[test]
fn get_desc_test() {
    const QUEUE_SIZE: usize = 2;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16 }, DummyClient());
    let heap = Heap::new(&memory);
    let mut server = DefaultQueueServer::new(&heap);
    server.init_queue(&queue).unwrap();
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
    let queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16 }, DummyClient());
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
    queue.set_desc_addr(desc_mem.info.base);
    queue.set_avail_addr(avail_mem.info.base);
    queue.set_used_addr(used_mem.info.base);


    assert_eq!(avail_mem.info.size, 4 + 10 * 2);
    SizedAccess::write(avail_mem.deref(), &avail_mem.info.base, &avail_ring);
    *queue.last_avail_idx.borrow_mut() = Wrapping(11);
    for pair in queue.avail_iter().unwrap().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + queue.last_avail_idx.borrow().0 as usize % queue.get_queue_size()], pair.1)
    }
    *queue.last_avail_idx.borrow_mut() = Wrapping(14);
    U16Access::write(avail_mem.deref(), &(avail_mem.info.base + 2), 16);
    for pair in queue.avail_iter().unwrap().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + queue.last_avail_idx.borrow().0 as usize % queue.get_queue_size()], pair.1)
    }
}

#[test]
fn add_to_queue_test() {
    const QUEUE_SIZE: usize = 10;
    let memory = GHEAP.alloc(1024, 16).unwrap();
    let queue = Queue::new(&memory, QueueSetting { max_queue_size: QUEUE_SIZE as u16 }, DummyClient());
    let heap = Heap::new(&memory);
    let mut server = DefaultQueueServer::new(&heap);
    server.init_queue(&queue).unwrap();
    for i in 0..(queue.get_queue_size() - 1) {
        let desc = queue.get_desc(i as u16).unwrap();
        assert_eq!(desc.next, i as u16 + 1)
    }

    let read_mem = heap.alloc(7, 1).unwrap();
    let write_mem = heap.alloc(6, 1).unwrap();

    let head = server.add_to_queue(&queue, vec![read_mem.deref()].as_slice(), vec![write_mem.deref()].as_slice()).unwrap();
    server.notify_queue(&queue, head).unwrap();

    let read_mem1 = heap.alloc(7, 1).unwrap();
    let write_mem1 = heap.alloc(6, 1).unwrap();
    let write_mem2 = heap.alloc(6, 1).unwrap();
    let write_mem3 = heap.alloc(6, 1).unwrap();

    let head = server.add_to_queue(&queue, vec![read_mem1.deref()].as_slice(), vec![write_mem1.deref(), write_mem2.deref(), write_mem3.deref()].as_slice()).unwrap();
    server.notify_queue(&queue, head).unwrap();

    let mut avail_iter = queue.avail_iter().unwrap();
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