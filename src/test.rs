use super::*;
use crate::model::*;
use crate::space::*;

#[test]
fn space_drop() {
    let space = Space::new();
    let heap = Heap::global();
    let region = space.add_region(String::from("region"), heap.alloc(9, 1));
    let &info = &region.info;
    let heap1 = Heap::new(&space.get_region(String::from("region")));
    let remap = Region::mmap(0x80000000, &space.get_region(String::from("region")));
    let remap2 = Region::mmap(0x10000000, &space.get_region(String::from("region")));
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(region);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(remap2);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    let region1 = heap1.alloc(2, 1);
    std::mem::drop(heap1);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    let remap3 = Region::mmap(0x10000000, &region1);
    std::mem::drop(region1);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(remap3);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    std::mem::drop(remap);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
    space.delete_region(String::from("region"));
    println!("{:?}", heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).collect::<Vec<MemInfo>>());
    assert_eq!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l| { l.car().unwrap() }).find(|i| { i == &info }), None);
}

#[test]
fn space_query() {
    let space = Space::new();
    let heap = Heap::global();
    let region = space.add_region(String::from("region"), heap.alloc(9, 1));
    let region2 = space.add_region(String::from("region2"), Region::mmap(0x80000000, &heap.alloc(9, 1)));
    let region3 = space.add_region(String::from("region3"), Region::mmap(0x10000000, &region));
    assert_eq!(space.get_region_by_addr(region2.info.base+8).info, region2.info);
    assert_eq!(space.get_region_by_addr(region3.info.base+2).info, region3.info);
}