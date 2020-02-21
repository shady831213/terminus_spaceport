use super::*;
// use crate::allocator::list::*;

#[test]
fn region_drop() {
    let heap = Heap::global();
    println!("{:?}",heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).collect::<Vec<AllocationInfo>>());
    let region = heap.alloc(9, 1);
    let &info = &region.info;
    let heap1 = Heap::new(&region);
    let remap = Region::mmap(0x80000000, &region);
    let remap2 = Region::mmap(0x10000000, &region);
    println!("{:?}",heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).collect::<Vec<AllocationInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(region);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(remap2);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    let region1 = heap1.alloc(2,1);
    std::mem::drop(heap1);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    let remap3 = Region::mmap(0x10000000, &region1);
    std::mem::drop(region1);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(remap3);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(remap);
    assert_eq!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
}

#[test]
fn region_access() {

    let heap = &Heap::global();
    {
        let region = heap.alloc(9, 1);
        let remap = Region::mmap(0x80000000, &region);
        let remap2 = Region::mmap(0x10000000, &region);
        U64Access::write(region.deref(), region.info.base, 0x5a5aa5a5aaaa5555);
        assert_eq!(U32Access::read(remap.deref(), remap.info.base), 0xaaaa5555);
        assert_eq!(U32Access::read(remap2.deref(), remap2.info.base + 4), 0x5a5aa5a5);
    }
}