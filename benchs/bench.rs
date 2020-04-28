#![feature(test)]
extern crate test;

use test::Bencher;

extern crate terminus_spaceport;

use terminus_spaceport::memory::region::*;
use rand::Rng;
use std::ops::Deref;
use terminus_spaceport::space::Space;

#[cfg(feature = "memprof")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const MAX_RND: usize = 1000000;

#[bench]
fn bench_model_access(b: &mut Bencher) {
    let region = GHEAP.alloc(0x1_0000_0000, 1).unwrap();
    let mut rng = rand::thread_rng();
    let mut addrs = vec![];
    for _ in 0..MAX_RND {
        addrs.push((rng.gen::<u64>() % 0x1_0000_0000 >> 3) << 3)
    }
    let mut i = 0;
    let mut get_addr = || {
        let data = addrs.get(i).unwrap();
        if i == MAX_RND - 1 {
            i = 0
        } else {
            i = i + 1
        }
        data
    };
    b.iter(|| {
        U64Access::write(region.deref(), get_addr(), 0xaa);
        U64Access::read(region.deref(), get_addr());
    });
    #[cfg(feature = "memprof")]
        unsafe { jemalloc_sys::malloc_stats_print(None, null_mut(), null()) };
}

#[bench]
fn bench_space_access(b: &mut Bencher) {
    let region = GHEAP.alloc(0x1_0000_0000, 1).unwrap();
    let region2 = GHEAP.lazy_alloc(0x000c0000, 1).unwrap();
    let region3 = GHEAP.lazy_alloc(0x0001000, 1).unwrap();

    let mut space = Space::new();
    space.add_region("memory", &Region::remap(0x8000_0000, &region)).unwrap();
    space.add_region("memory2", &Region::remap(0x200_0000, &region2)).unwrap();
    space.add_region("memory3", &Region::remap(0x2000_0000, &region3)).unwrap();

    let mut rng = rand::thread_rng();
    let mut addrs = vec![];
    for _ in 0..MAX_RND {
        addrs.push(0x8000_0000 + ((rng.gen::<u64>() % 0x1_0000_0000 >> 3) << 3))
    }
    let mut i = 0;
    let mut get_addr = || {
        let data = addrs.get(i).unwrap();
        if i == MAX_RND - 1 {
            i = 0
        } else {
            i = i + 1
        }
        data
    };
    b.iter(|| {
        space.write_bytes(get_addr(), &0xaau64.to_le_bytes()).unwrap();
        let mut data = [0u8; 8];
        space.read_bytes(get_addr(), &mut data).unwrap();
    });
    #[cfg(feature = "memprof")]
        unsafe { jemalloc_sys::malloc_stats_print(None, null_mut(), null()) };
}