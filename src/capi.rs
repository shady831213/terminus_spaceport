use crate::{Allocator, LockedAllocator};

#[no_mangle]
extern "C" fn dm_new_allocator(base: u64, size: u64) -> *const Allocator {
    Box::into_raw(Box::new(Allocator::new(base, size)))
}

#[no_mangle]
//unsafe raw pointer style
extern "C" fn dm_alloc(a: *mut Allocator, size: u64, align: u64) -> u64 {
    unsafe {
        let info = (&mut *a).alloc(size, align);
        if let Some(i) = info {
            i.base
        } else {
            panic!("oom!")
        }
    }
}

#[no_mangle]
//unsafe raw pointer style
extern "C" fn dm_free(a: *mut Allocator, addr: u64) {
    unsafe {
        (&mut *a).free(addr);
    }
}

#[no_mangle]
extern "C" fn dm_new_locked_allocator(base: u64, size: u64) -> *const LockedAllocator {
    Box::into_raw(Box::new(LockedAllocator::new(base, size)))
}

#[no_mangle]
//safe reference style
extern "C" fn dm_locked_alloc(a: &LockedAllocator, size: u64, align: u64) -> u64 {
    let info = a.alloc(size, align);
    if let Some(i) = info {
        i.base
    } else {
        panic!("oom!")
    }
}

#[no_mangle]
//safe reference style
extern "C" fn dm_locked_free(a: &LockedAllocator, addr: u64) {
    a.free(addr);
}