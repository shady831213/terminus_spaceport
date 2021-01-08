use crate::memory::region::Heap;
use crate::memory::region::Region;
use crate::memory::region::GHEAP;
use crate::memory::MemInfo;
use crate::space::*;

#[test]
fn space_drop() {
    let mut space = Space::new();
    let heap = &GHEAP;
    let region = space
        .add_region("region", &heap.alloc(188, 1024).unwrap())
        .unwrap();
    let &info = &region.info;
    let heap1 = Box::new(Heap::new(&space.get_region("region").unwrap()));
    let remap = Box::new(Region::remap(
        0x80000000,
        &space.get_region("region").unwrap(),
    ));
    let remap2 = Region::remap(0x10000000, &space.get_region("region").unwrap());
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    std::mem::drop(region);
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    std::mem::drop(remap2);
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    let region1 = heap1.alloc(2, 1).unwrap();
    std::mem::drop(heap1);
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    let remap3 = Region::remap(0x10000000, &region1);
    std::mem::drop(region1);
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    std::mem::drop(remap3);
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    std::mem::drop(remap);
    assert_ne!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
    space.delete_region("region");
    println!(
        "{:?}",
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .collect::<Vec<MemInfo>>()
    );
    assert_eq!(
        heap.allocator
            .lock()
            .unwrap()
            .alloced_blocks
            .iter()
            .map(|l| { l.car().unwrap() })
            .find(|i| { i == &info }),
        None
    );
}

#[test]
fn space_query() {
    let mut space = Space::new();
    let heap = &GHEAP;
    let region = space
        .add_region("region", &heap.alloc(9, 1).unwrap())
        .unwrap();
    let region2 = space
        .add_region(
            "region2",
            &Region::remap(0x80000000, &heap.alloc(9, 1).unwrap()),
        )
        .unwrap();
    let region3 = space
        .add_region("region3", &Region::remap(0x10000000, &region))
        .unwrap();
    assert_eq!(
        space.get_region_by_addr(&region2.info.base).unwrap().info,
        region2.info
    );
    assert_eq!(
        space
            .get_region_by_addr(&(region3.info.base + 2))
            .unwrap()
            .info,
        region3.info
    );
}
