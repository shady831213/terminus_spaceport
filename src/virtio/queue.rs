use crate::memory::{Region, Heap, SizedAccess, U16Access};
use std::sync::Arc;
use std::{mem, result};
use std::ops::Deref;
use std::cmp::min;
use std::num::Wrapping;
use std::marker::{PhantomData, Sized};

enum Error {
    InvalidIdx
}

type Result<T> = result::Result<T, Error>;

#[derive(Copy, Clone)]
pub struct QueueSetting {
    max_num: u16,
    manual_recv: bool,
}

#[derive(Debug, Eq, PartialEq)]
struct Desc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

impl Desc {
    fn empty() -> Desc {
        Desc {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

struct RingHeader {
    flags: u16,
    idx: u16,
}

type RingAvailElem = u16;

struct Ring<T: ?Sized> {
    info: RingHeader,
    ring: T
}


struct RingUsedElem {
    id: u32,
    len: u32,
}

pub struct Queue {
    setting: QueueSetting,
    memory: Arc<Region>,
    ready: bool,
    num: u16,
    last_avail_idx: u16,
    desc_addr: u64,
    avail_addr: u64,
    used_addr: u64,
}

impl Queue {
    pub fn new(memory: &Arc<Region>, setting: QueueSetting) -> Queue {
        // assert!(max_num.is_power_of_two());
        let max_num = setting.max_num;
        Queue {
            setting,
            memory: Arc::clone(memory),
            ready: false,
            num: max_num,
            last_avail_idx: 0,
            desc_addr: 0,
            avail_addr: 0,
            used_addr: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ready = false;
        self.last_avail_idx = 0;
        self.num = self.setting.max_num;
        self.desc_addr = 0;
        self.avail_addr = 0;
        self.used_addr = 0;
    }

    fn get_num(&self) -> u16 {
        min(self.num, self.setting.max_num)
    }

    fn check_idx(&self, idx: u16) -> Result<()> {
        if idx >= self.get_num() {
            Err(Error::InvalidIdx)
        } else {
            Ok(())
        }
    }

    fn desc_addr(&self, idx: u16) -> u64 {
        if let Err(_) = self.check_idx(idx) {
            panic!(format!("invalid desc idx! {}", idx))
        }
        self.desc_addr + (idx as usize * mem::size_of::<Desc>()) as u64
    }

    fn get_desc(&self, idx: u16) -> Desc {
        let mut desc = Desc::empty();
        SizedAccess::read(self.memory.deref(), self.desc_addr(idx), &mut desc);
        desc
    }

    fn add_desc(&self, idx: u16, desc: &Desc) {
        SizedAccess::write(self.memory.deref(), self.desc_addr(idx), desc)
    }

    fn avail_iter<'a>(&'a self) -> AvailIter<'a> {
        let mut header = RingHeader { flags: 0, idx: 0 };
        SizedAccess::read(self.memory.deref(), self.avail_addr, &mut header);
        AvailIter::new(self.memory.deref(),
                       self.avail_addr,
                       Wrapping(header.idx),
                       Wrapping(self.last_avail_idx), self.num,
                       PhantomData)
    }
}

struct AvailIter<'a> {
    memory: &'a Region,
    avail_addr: u64,
    end_idx: Wrapping<u16>,
    next_idx: Wrapping<u16>,
    num: u16,
    marker: PhantomData<&'a Queue>,
}

impl<'a> AvailIter<'a> {
    fn new(memory: &'a Region,
           avail_addr: u64,
           end_idx: Wrapping<u16>,
           next_idx: Wrapping<u16>,
           num: u16,
           marker: PhantomData<&'a Queue>) -> AvailIter<'a> {
        AvailIter {
            memory,
            avail_addr,
            end_idx,
            next_idx,
            num,
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

        let ring_elem_addr = self.avail_addr + mem::size_of::<RingHeader>() as u64 + (self.next_idx.0 % self.num) as u64 * mem::size_of::<RingAvailElem>() as u64;
        let mut desc_idx = 0 as RingAvailElem;
        SizedAccess::read(self.memory, ring_elem_addr, &mut desc_idx);
        self.next_idx += Wrapping(1);
        Some(desc_idx)
    }
}


#[test]
fn get_desc_test() {
    let memory = Heap::global().alloc(32, 16).unwrap();
    let queue = Queue::new(&memory, QueueSetting { max_num: 2, manual_recv: false });
    queue.add_desc(0, &Desc {
        addr: 0xa5a5,
        len: 0x5a5a,
        flags: 0xdead,
        next: 0xbeaf,
    });
    assert_eq!(queue.get_desc(0), Desc {
        addr: 0xa5a5,
        len: 0x5a5a,
        flags: 0xdead,
        next: 0xbeaf,
    });
}

#[test]
fn avail_test() {
    let memory = Heap::global().alloc(1024, 16).unwrap();
    let mut queue = Queue::new(&memory, QueueSetting { max_num: 10, manual_recv: false });
    let heap = Heap::new(&memory);
    let mut avail_ring:Ring<[RingAvailElem;10]> = Ring {
        info: RingHeader {
            flags: 0,
            idx: 14,
        },
        ring: [0 as RingAvailElem;10],
    };
    avail_ring.ring[1] = 3;
    avail_ring.ring[2] = 5;
    avail_ring.ring[3] = 8;
    avail_ring.ring[4] = 7;
    avail_ring.ring[5] = 1;


    let avail_mem = heap.alloc(mem::size_of_val(&avail_ring) as u64, 32).unwrap();
    assert_eq!(avail_mem.info.size, 4 + 10*2);
    SizedAccess::write(avail_mem.deref(), avail_mem.info.base, &avail_ring);
    queue.last_avail_idx = 11;
    for pair in queue.avail_iter().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + (queue.last_avail_idx % queue.num) as usize], pair.1)
    }
    queue.last_avail_idx = 14;
    U16Access::write(avail_mem.deref(), avail_mem.info.base + 2, 16);
    for pair in queue.avail_iter().enumerate() {
        assert_eq!(avail_ring.ring[pair.0 + (queue.last_avail_idx % queue.num) as usize], pair.1)
    }
}