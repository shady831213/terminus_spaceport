use crate::memory::{Region, Heap, SizedAccess, U16Access};
use std::sync::Arc;
use std::{mem, result};
use std::ops::Deref;
use std::cmp::min;
use std::num::Wrapping;
use std::marker::{PhantomData, Sized};

const DESC_F_NEXT: u16 = 0x1;
const DESC_F_WRITE: u16 = 0x2;

enum Error {
    InvalidIdx
}

type Result<T> = result::Result<T, Error>;

#[derive(Copy, Clone)]
pub struct QueueSetting {
    max_queue_size: u16,
    manual_recv: bool,
}

#[derive(Debug, Eq, PartialEq)]
struct DescMeta {
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

type RingAvailMetaElem = u16;

struct RingMeta<T: ?Sized> {
    info: RingMetaHeader,
    ring: T,
}


struct RingUsedMetaElem {
    id: u32,
    len: u32,
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

    fn get_queue_size(&self) -> u16 {
        min(self.queue_size, self.setting.max_queue_size)
    }

    fn check_idx(&self, idx: u16) -> Result<()> {
        if idx >= self.get_queue_size() {
            Err(Error::InvalidIdx)
        } else {
            Ok(())
        }
    }

    fn desc_addr(&self, idx: u16) -> u64 {
        if let Err(_) = self.check_idx(idx) {
            panic!(format!("invalid desc idx! {}", idx))
        }
        self.desc_addr + (idx as usize * mem::size_of::<DescMeta>()) as u64
    }

    fn get_desc<'a>(&'a self, idx: u16) -> DescHead<'a> {
        DescHead::new(self.memory.deref(), self.desc_addr, idx, self.get_queue_size(),1, PhantomData)
    }

    fn add_desc(&self, idx: u16, desc: &DescMeta) {
        SizedAccess::write(self.memory.deref(), self.desc_addr(idx), desc)
    }

    fn avail_iter<'a>(&'a self) -> AvailIter<'a> {
        let mut header = RingMetaHeader { flags: 0, idx: 0 };
        SizedAccess::read(self.memory.deref(), self.avail_addr, &mut header);
        AvailIter::new(self.memory.deref(),
                       self.avail_addr,
                       Wrapping(header.idx),
                       Wrapping(self.last_avail_idx), self.get_queue_size(),
                       PhantomData)
    }
}

pub struct AvailIter<'a> {
    memory: &'a Region,
    avail_addr: u64,
    end_idx: Wrapping<u16>,
    next_idx: Wrapping<u16>,
    queue_size: u16,
    marker: PhantomData<&'a Queue>,
}

impl<'a> AvailIter<'a> {
    fn new(memory: &'a Region,
           avail_addr: u64,
           end_idx: Wrapping<u16>,
           next_idx: Wrapping<u16>,
           queue_size: u16,
           marker: PhantomData<&'a Queue>) -> AvailIter<'a> {
        AvailIter {
            memory,
            avail_addr,
            end_idx,
            next_idx,
            queue_size,
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

        let ring_elem_addr = self.avail_addr + mem::size_of::<RingMetaHeader>() as u64 + (self.next_idx.0 % self.queue_size) as u64 * mem::size_of::<RingAvailMetaElem>() as u64;
        let mut desc_idx = 0 as RingAvailMetaElem;
        SizedAccess::read(self.memory, ring_elem_addr, &mut desc_idx);
        self.next_idx += Wrapping(1);
        Some(desc_idx)
    }
}

type DescHead<'a> = Desc<'a, Queue>;

pub struct Desc<'a, T> {
    meta: DescMeta,
    memory: &'a Region,
    desc_addr: u64,
    queue_size: u16,
    ttl: u16,
    marker: PhantomData<&'a T>,
}

impl<'a, T> Desc<'a, T> {
    fn new(memory: &'a Region,
           desc_addr: u64,
           idx:u16,
           queue_size: u16,
           ttl: u16,
           marker: PhantomData<&'a T>) -> Desc<'a, T> {
        let mut desc = Desc {
            meta:DescMeta::empty(),
            memory,
            desc_addr,
            queue_size,
            ttl,
            marker,
        };
        SizedAccess::read(memory, desc.meta_addr(idx), &mut desc.meta);
        desc
    }

    pub fn last(&self) -> bool {
        self.meta.flags & DESC_F_NEXT == 0 || self.ttl >= self.queue_size
    }

    pub fn writeable(&self) -> bool {
        self.meta.flags & DESC_F_WRITE != 0
    }

    fn check_idx(&self,idx:u16) -> Result<()> {
        if idx >= self.queue_size {
            Err(Error::InvalidIdx)
        } else {
            Ok(())
        }
    }

    fn meta_addr(&self,idx:u16) -> u64 {
        if let Err(_) = self.check_idx(idx) {
            panic!(format!("invalid desc idx! {}", idx))
        }
        self.desc_addr + (idx as usize * mem::size_of::<DescMeta>()) as u64
    }

    pub fn next(&'a self) -> Option<Desc::<'a, Self>> {
        if self.last() {
            None
        } else {
            Some(Desc::<'a, Self>::new(self.memory, self.desc_addr, self.meta.next, self.queue_size, self.ttl+1, PhantomData))
        }
    }
}


#[test]
fn get_desc_test() {
    let memory = Heap::global().alloc(32, 16).unwrap();
    let queue = Queue::new(&memory, QueueSetting { max_queue_size: 2, manual_recv: false });
    queue.add_desc(0, &DescMeta {
        addr: 0xa5a5,
        len: 0x5a5a,
        flags: 0,
        next: 0xbeaf,
    });
    let desc = queue.get_desc(0);
    assert_eq!(desc.meta, DescMeta {
        addr: 0xa5a5,
        len: 0x5a5a,
        flags: 0,
        next: 0xbeaf,
    });
    assert_eq!(desc.next().is_none(), true);
}

#[test]
fn avail_test() {
    let memory = Heap::global().alloc(1024, 16).unwrap();
    let mut queue = Queue::new(&memory, QueueSetting { max_queue_size: 10, manual_recv: false });
    let heap = Heap::new(&memory);
    let mut avail_ring: RingMeta<[RingAvailMetaElem; 10]> = RingMeta {
        info: RingMetaHeader {
            flags: 0,
            idx: 14,
        },
        ring: [0 as RingAvailMetaElem; 10],
    };
    avail_ring.ring[1] = 3;
    avail_ring.ring[2] = 5;
    avail_ring.ring[3] = 8;
    avail_ring.ring[4] = 7;
    avail_ring.ring[5] = 1;


    let avail_mem = heap.alloc(mem::size_of_val(&avail_ring) as u64, 32).unwrap();
    assert_eq!(avail_mem.info.size, 4 + 10 * 2);
    SizedAccess::write(avail_mem.deref(), avail_mem.info.base, &avail_ring);
    queue.last_avail_idx = 11;
    for pair in queue.avail_iter().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + (queue.last_avail_idx % queue.get_queue_size()) as usize], pair.1)
    }
    queue.last_avail_idx = 14;
    U16Access::write(avail_mem.deref(), avail_mem.info.base + 2, 16);
    for pair in queue.avail_iter().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + (queue.last_avail_idx % queue.get_queue_size()) as usize], pair.1)
    }
}