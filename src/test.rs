use super::*;
use crate::memory::*;
use crate::space::*;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ops::Deref;
use std::thread;
use std::thread::sleep;
use std::time::Duration;


#[test]
fn space_drop() {
    let mut space = Space::new();
    let heap = Heap::global();
    let region = space.add_region("region", &heap.alloc(9, 1)).unwrap();
    let &info = &region.info;
    let heap1 = Box::new(Heap::new(&space.get_region("region").unwrap()));
    let remap = Box::new(Region::remap(0x80000000, &space.get_region("region").unwrap()));
    let remap2 = Region::remap(0x10000000, &space.get_region("region").unwrap());
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(region);
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(remap2);
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    let region1 = heap1.alloc(2, 1);
    std::mem::drop(heap1);
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    let remap3 = Region::remap(0x10000000, &region1);
    std::mem::drop(region1);
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(remap3);
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(remap);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    space.delete_region("region");
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_eq!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
}

#[test]
fn space_query() {
    let mut space = Space::new();
    let heap = Heap::global();
    let region = space.add_region("region", &heap.alloc(9, 1)).unwrap();
    let region2 = space.add_region("region2", &Region::remap(0x80000000, &heap.alloc(9, 1))).unwrap();
    let region3 = space.add_region("region3", &Region::remap(0x10000000, &region)).unwrap();
    assert_eq!(space.get_region_by_addr(region2.info.base + 8).unwrap().info, region2.info);
    assert_eq!(space.get_region_by_addr(region3.info.base + 2).unwrap().info, region3.info);

    let send_thread = {
        thread::spawn(move || {
            for i in 0..10 {
                U8Access::write(region2.deref(), region2.info.base + 8, i);
            }
        })
    };
    send_thread.join().unwrap();

    println!("{}", space.to_string());
}

#[derive_io(U8)]
struct TestIODevice {
    tx: Mutex<Sender<u8>>,
    rx: Mutex<Receiver<u8>>,
}

impl TestIODevice {
    fn new(tx: Sender<u8>, rx: Receiver<u8>) -> TestIODevice {
        TestIODevice {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

impl U8Access for TestIODevice {
    fn write(&self, addr: u64, data: u8) {
        let tx = self.tx.lock().unwrap();
        tx.send(addr as u8).unwrap();
        sleep(Duration::from_nanos(300));
        tx.send(data).unwrap();
    }

    fn read(&self, _: u64) -> u8 {
        self.rx.lock().unwrap().recv().unwrap()
    }
}

#[test]
fn simple_device() {
    let space = SpaceTable::global().get_space("");
    let (recv_tx, recv_rx) = channel();
    let (send_tx, send_rx) = channel();
    let (stop_tx, stop_rx) = channel::<()>();
    let region = Region::io(0, 20, Box::new(TestIODevice::new(recv_tx, send_rx)));
    space.write().unwrap().add_region("testIO", &region).unwrap();

    thread::spawn(move || {
        for i in 0..10 {
            sleep(Duration::from_micros(1));
            U8Access::write(SpaceTable::global().get_space("").read().unwrap().get_region("testIO").unwrap().deref(), 10 - (i as u64), i);
        }
    });

    thread::spawn(move || {
        for i in 0..10 {
            sleep(Duration::from_micros(1));
            U8Access::write(SpaceTable::global().get_space("").read().unwrap().get_region("testIO").unwrap().deref(), 10 - (i as u64), i);
        }
    });

    let recv_thread = {
        thread::spawn(move || {
            for _ in 0..40 {
                U8Access::read(SpaceTable::global().get_space("").read().unwrap().get_region("testIO").unwrap().deref(), 0);
            }
        })
    };

    let loopback_tread = {
        thread::spawn(move || {
            loop {
                match stop_rx.try_recv() {
                    Ok(_) => {
                        break;
                    }
                    _ => {
                        match recv_rx.try_recv() {
                            Ok(v) => { send_tx.send(v).unwrap(); }
                            _ => {}
                        }
                    }
                }
            }
        })
    };

    recv_thread.join().unwrap();
    stop_tx.send(()).unwrap();
    loopback_tread.join().unwrap();
}