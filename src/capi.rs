use crate::allocator::{Allocator, LockedAllocator};
use std::any::Any;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use crate::space::Space;
use std::sync::{Arc, RwLock};
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
extern "C" fn dm_alloc_addr(a: &mut Box<dyn Any>, size: u64, align: u64) -> u64 {
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
extern "C" fn dm_free_addr(a: *mut c_void, addr: u64) {
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


#[no_mangle]
extern "C" fn dm_new_space() -> *const RwLock<Space> {
    Box::into_raw(Box::new(RwLock::new(Space::new())))
}

#[no_mangle]
extern "C" fn __dm_add_region(space: &RwLock<Space>, name: *const c_char, region: &Box<Arc<Region>>) -> *const Box<Arc<Region>> {
    to_c_ptr(space.write().unwrap().add_region(unsafe { CStr::from_ptr(name).to_str().unwrap() }, region.deref()))
}

#[no_mangle]
extern "C" fn __dm_clean_region(space: &RwLock<Space>, name: *const c_char, region: *const Box<Arc<Region>>) {
    space.write().unwrap().clean(unsafe { CStr::from_ptr(name).to_str().unwrap() }, region)
}

#[no_mangle]
extern "C" fn __dm_get_region(space: &RwLock<Space>, name: *const c_char) -> *const Box<Arc<Region>> {
    to_c_ptr(space.read().unwrap().get_region(unsafe { CStr::from_ptr(name).to_str().unwrap() }))
}

#[no_mangle]
extern "C" fn dm_delete_region(space: &RwLock<Space>, name: *const c_char) {
    space.write().unwrap().delete_region(unsafe { CStr::from_ptr(name).to_str().unwrap() })
}

#[no_mangle]
extern "C" fn dm_alloc_region(heap: *const Box<Arc<Heap>>, size: u64, align: u64) -> *const Box<Arc<Region>> {
    to_c_ptr(unsafe {
        if heap.is_null() {
            // println!("alloc from global heap!");
            Heap::global().alloc(size, align)
        } else {
            let p = heap.as_ref().unwrap();
            let region = p.alloc(size, align);
            // println!("alloc {:?} from heap!", region.info);
            region
        }
    })
}

#[no_mangle]
extern "C" fn dm_free_region(region: *const Box<Arc<Region>>) {
    std::mem::drop(unsafe { region.read() })
}

#[no_mangle]
extern "C" fn dm_heap(region: &Box<Arc<Region>>) -> *const Box<Arc<Heap>> {
    Box::into_raw(Box::new(Box::new(Heap::new(region.deref()))))
}

#[no_mangle]
extern "C" fn dm_free_heap(heap: *const Box<Arc<Heap>>) {
    std::mem::drop(unsafe { heap.read() })
}

#[no_mangle]
extern "C" fn dm_map_region(region: &Box<Arc<Region>>, base: u64) -> *const Box<Arc<Region>> {
    to_c_ptr(Region::mmap(base, region.deref()))
}

fn to_c_ptr(obj: Arc<Region>) -> *const Box<Arc<Region>> {
    Box::into_raw(Box::new(Box::new(obj)))
}
