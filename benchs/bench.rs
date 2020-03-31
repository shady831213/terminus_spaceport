#![feature(test)]
extern crate test;

use test::Bencher;
extern crate terminus_spaceport;

use terminus_spaceport::memory::*;
use rand::Rng;
use std::ops::Deref;
use std::ptr::{null, null_mut};

#[cfg(feature = "memprof")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const MAX_RND:usize = 1000000;
#[bench]
fn bench_model_access(b: &mut Bencher) {
    let region = GHEAP.alloc(0x1_0000_0000, 1).unwrap();
    let mut rng = rand::thread_rng();
    let mut addrs = vec![];
    for _ in 0 .. MAX_RND {
        addrs.push(rng.gen::<u64>() % 0x1_0000_0000)
    }
    let mut i = 0;
    let mut get_addr = || {
        let data = addrs.get(i).unwrap();
        if i == MAX_RND - 1 {
            i = 0
        } else {
            i = i + 1
        }
        *data
    };
    b.iter(|| {
        U64Access::write(region.deref(), get_addr(), 0xaa).unwrap();
        U64Access::read(region.deref(), get_addr()).unwrap();
    });
    #[cfg(feature = "memprof")]
    unsafe { jemalloc_sys::malloc_stats_print(None, null_mut(), null()) };
}