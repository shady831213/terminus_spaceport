use super::*;

#[test]
fn region_drop() {
    let heap = &GHEAP;
    println!("{:?}",heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).collect::<Vec<MemInfo>>());
    let region = heap.alloc(9, 1).unwrap();
    let &info = &region.info;
    let heap1 = Heap::new(&region);
    let remap = Region::remap(0x80000000, &region);
    let remap2 = Region::remap(0x10000000, &region);
    println!("{:?}",heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).collect::<Vec<MemInfo>>());
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(region);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(remap2);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    let region1 = heap1.alloc(2,1).unwrap();
    std::mem::drop(heap1);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    let remap3 = Region::remap(0x10000000, &region1);
    std::mem::drop(region1);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(remap3);
    assert_ne!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
    std::mem::drop(remap);
    assert_eq!(heap.allocator.lock().unwrap().alloced_blocks.iter().map(|l|{l.car().unwrap()}).find(|i|{i == &info}), None);
}

#[test]
fn region_access() {

    let heap = &&GHEAP;
    {
        let region = heap.alloc(9, 8).unwrap();
        let remap = Region::remap(0x80000000, &region);
        let remap2 = Region::remap_partial(0x10000000, &region, 0, 5);
        let remap3 = Region::remap_partial(0x20000000, &region, 5, 4);
        U64Access::write(region.deref(), region.info.base, 0x5a5aa5a5aaaa5555);
        U8Access::write(region.deref(), region.info.base+8, 0xab);
        assert_eq!(U32Access::read(remap.deref(), remap.info.base), 0xaaaa5555);
        assert_eq!(U8Access::read(remap2.deref(), remap2.info.base+4), 0xa5);
        assert_eq!(U32Access::read(remap3.deref(), remap3.info.base), 0xab5a5aa5);
        U32Access::write(remap.deref(), remap.info.base, 0xbeefdead);
        U8Access::write(remap2.deref(), remap2.info.base + 4, 0xef);
        U32Access::write(remap3.deref(), remap3.info.base, 0xccdeadbe);
        assert_eq!(U64Access::read(region.deref(), region.info.base), 0xdeadbeefbeefdead);
        assert_eq!(U8Access::read(region.deref(), region.info.base+8), 0xcc);
    }
}