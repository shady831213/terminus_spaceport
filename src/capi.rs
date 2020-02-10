use crate::{Allocator, LockedAllocator};

#[no_mangle]
extern "C" fn new_allocator(base: u64, size: u64) -> *const Allocator {
    Box::into_raw(Box::new(Allocator::new(base, size)))
}

#[no_mangle]
extern "C" fn alloc(a: &mut Allocator, size: u64, align: u64) -> u64 where {
    let info = a.alloc(size, align);
    if let Some(i) = info {
        i.base
    } else {
        panic!("oom!")
    }
}

#[no_mangle]
extern "C" fn new_locked_allocator(base: u64, size: u64) -> *const LockedAllocator {
    Box::into_raw(Box::new(LockedAllocator::new(base, size)))
}

#[no_mangle]
extern "C" fn locked_alloc(a: &LockedAllocator, size: u64, align: u64) -> u64 {
    let info = a.alloc(size, align);
    if let Some(i) = info {
        i.base
    } else {
        panic!("oom!")
    }
}