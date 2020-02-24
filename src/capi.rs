use crate::allocator::{Allocator, LockedAllocator};
use std::any::Any;
use std::ffi::c_void;
use crate::space::Space;
use std::sync::Arc;
use crate::model::*;
use std::ops::Deref;


#[no_mangle]
extern "C" fn dm_new_allocator(base: u64, size: u64) -> *const c_void {
    Box::into_raw(Box::new(Box::new(Allocator::new(base, size)) as Box<dyn Any>)) as *const c_void
}

#[no_mangle]
extern "C" fn dm_new_locked_allocator(base: u64, size: u64) -> *const c_void {
    Box::into_raw(Box::new(Box::new(LockedAllocator::new(base, size)) as Box<dyn Any>)) as *const c_void
}

#[no_mangle]
//safe pointer style
extern "C" fn dm_alloc(a: &mut Box<dyn Any>, size: u64, align: u64) -> u64 {
    let info = if let Some(allocator) = a.downcast_mut::<Allocator>() {
        allocator.alloc(size, align)
    } else if let Some(allocator) = a.downcast_mut::<LockedAllocator>() {
        allocator.alloc(size, align)
    } else {
        panic!("wrong type!allocator should be create by dm_new_allocator or dm_new_locked_allocator!")
    };

    if let Some(i) = info {
        i.base
    } else {
        panic!("oom!")
    }
}

#[no_mangle]
//unsafe raw pointer style
extern "C" fn dm_free(a: *mut c_void, addr: u64) {
    let abox = unsafe {
        &mut *(a as *mut Box<dyn Any>)
    };

    if let Some(allocator) = abox.downcast_mut::<Allocator>() {
        allocator.free(addr)
    } else if let Some(allocator) = abox.downcast_mut::<LockedAllocator>() {
        allocator.free(addr)
    } else {
        panic!("wrong type!allocator should be create by dm_new_allocator or dm_new_locked_allocator!")
    }
}


// #[no_mangle]
// extern "C" fn dm_new_space() -> *const Space {
//     Box::into_raw(Box::new(Space::new()))
// }
//
// #[no_mangle]
// extern "C" fn dm_add_region(space: &Box<Space>, name: &str, region: &Box<Arc<Region>>) -> *const Arc<Region> {
//     Box::into_raw(Box::new(space.add_region(String::from(name), region)))
// }
//
// #[no_mangle]
// extern "C" fn dm_get_region(space: &Box<Space>, name: &str) -> *const Arc<Region> {
//     Box::into_raw(Box::new(space.get_region(String::from(name))))
// }
//
// #[no_mangle]
// extern "C" fn dm_delete_region(space: &Box<Space>, name: &str) {
//     space.delete_region(String::from(name))
// }
//
// #[no_mangle]
// extern "C" fn dm_region_alloc(heap: *const Arc<Heap>, base: u64, size: u64) -> *const Arc<Region> {
//     Box::into_raw(Box::new(unsafe {
//        if heap.is_null() {
//            Heap::global().alloc(base, size)
//        } else {
//            Arc::from_raw(heap).alloc(base, size)
//        }
//     }))
// }
//
// #[no_mangle]
// extern "C" fn dm_region_free(region: Box<Arc<Region>>) {
//     std::mem::drop(region)
// }
//
// #[no_mangle]
// extern "C" fn dm_new_heap(region: &Box<Arc<Region>>) -> *const Arc<Heap> {
//     Box::into_raw(Box::new(Heap::new(region)))
// }
//
// #[no_mangle]
// extern "C" fn dm_mmap(region: &Box<Arc<Region>>, base: u64) -> *const Arc<Region> {
//     Box::into_raw(Box::new(Region::mmap(base, region)))
// }