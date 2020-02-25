use crate::allocator::{Allocator, LockedAllocator};
use std::any::Any;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
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
extern "C" fn dm_new_space() -> *const Space {
    Box::into_raw(Box::new(Space::new()))
}

#[no_mangle]
extern "C" fn dm_add_region(space: &mut Space, name: *const c_char, region: &Box<dyn Any>) {
    space.add_region(unsafe { CStr::from_ptr(name).to_str().unwrap() }, get_region(region))
}

#[no_mangle]
extern "C" fn dm_get_region(space: &'static Space, name: *const c_char) -> *const c_void {
    to_c_void(space.get_region(unsafe { CStr::from_ptr(name).to_str().unwrap() }))
}

#[no_mangle]
extern "C" fn dm_delete_region(space: &mut Space, name: *const c_char) {
    space.delete_region(unsafe { CStr::from_ptr(name).to_str().unwrap() })
}

#[no_mangle]
extern "C" fn dm_alloc_region(heap: *const Box<Arc<Heap>>, size: u64, align: u64) -> *const c_void {
    to_c_void(unsafe {
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
extern "C" fn dm_free_region(ptr: *const Box<dyn Any>) {
    let region = unsafe { ptr.read() };
    if let Some(_) = region.downcast_ref::<Arc<Region>>() {
        std::mem::drop(region)
    } else {
        panic!("wrong type!only Arc<Region> can free!")
    }
}

#[no_mangle]
extern "C" fn dm_heap(region: &Box<dyn Any>) -> *const Box<Arc<Heap>> {
    Box::into_raw(Box::new(Box::new(Heap::new(get_region(region)))))
}

#[no_mangle]
extern "C" fn dm_free_heap(heap: *const Box<Arc<Heap>>) {
    std::mem::drop(unsafe { heap.read() })
}

#[no_mangle]
extern "C" fn dm_map_region(region: &Box<dyn Any>, base: u64) -> *const c_void {
    to_c_void(Region::mmap(base, get_region(region)))
}

fn to_c_void<T:'static>(obj: T) -> *const c_void {
    Box::into_raw(Box::new(Box::new(obj) as Box<dyn Any>)) as *const c_void
}

fn get_region(ptr: &Box<dyn Any>) -> &Arc<Region> {
    if let Some(region) = ptr.downcast_ref::<&Arc<Region>>() {
        region.deref()
    } else if let Some(region) = ptr.downcast_ref::<Arc<Region>>() {
        region
    } else {
        panic!("wrong type!region should be &Arc<Region> or Arc<Region>!")
    }
}