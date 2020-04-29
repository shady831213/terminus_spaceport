use std::any::Any;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use crate::space::Space;
use crate::memory::region::*;
use crate::memory::allocator::*;
use crate::memory::MemInfo;
use std::ops::Deref;
use std::rc::Rc;


#[no_mangle]
extern "C" fn __ts_new_allocator(base: u64, size: u64) -> *const c_void {
    Box::into_raw(Box::new(Box::new(Allocator::new(base, size)) as Box<dyn Any>)) as *const c_void
}

#[no_mangle]
extern "C" fn __ts_new_locked_allocator(base: u64, size: u64) -> *const c_void {
    Box::into_raw(Box::new(Box::new(LockedAllocator::new(base, size)) as Box<dyn Any>)) as *const c_void
}

#[no_mangle]
//safe pointer style
extern "C" fn __ts_alloc_addr(a: &mut Box<dyn Any>, size: u64, align: u64) -> u64 {
    let info = if let Some(allocator) = a.downcast_mut::<Allocator>() {
        allocator.alloc(size, align)
    } else if let Some(allocator) = a.downcast_mut::<LockedAllocator>() {
        allocator.alloc(size, align)
    } else {
        panic!("wrong type!allocator should be create by ts_new_allocator or ts_new_locked_allocator!")
    };

    if let Some(i) = info {
        i.base
    } else {
        panic!("oom!")
    }
}

#[no_mangle]
//unsafe raw pointer style
extern "C" fn __ts_free_addr(a: *mut c_void, addr: u64) {
    let abox = unsafe {
        &mut *(a as *mut Box<dyn Any>)
    };

    if let Some(allocator) = abox.downcast_mut::<Allocator>() {
        allocator.free(addr)
    } else if let Some(allocator) = abox.downcast_mut::<LockedAllocator>() {
        allocator.free(addr)
    } else {
        panic!("wrong type!allocator should be create by ts_new_allocator or ts_new_locked_allocator!")
    }
}


#[no_mangle]
extern "C" fn __ts_space() -> *mut Space {
    Box::into_raw(Box::new(Space::new()))
}

#[no_mangle]
extern "C" fn __ts_add_region(space: &mut Space, name: *const c_char, region: &Box<Rc<Region>>) -> *const Box<Rc<Region>> {
    let name = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    match space.add_region(name, region.deref()) {
        Ok(r) => to_c_ptr(r),
        Err(e) => panic!(format!("{:?}", e))
    }
}

#[no_mangle]
extern "C" fn __ts_clean_region(space: &mut Space, name: *const c_char, region: *const Box<Rc<Region>>) {
    space.clean(unsafe { CStr::from_ptr(name).to_str().unwrap() }, region)
}

#[no_mangle]
extern "C" fn __ts_get_region(space: &Space, name: *const c_char) -> *const Box<Rc<Region>> {
    let name = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    if let Some(r) = space.get_region(name) {
        to_c_ptr(r)
    } else {
        panic!(format!("no region {}", name))
    }
}

#[no_mangle]
extern "C" fn __ts_delete_region(space: &mut Space, name: *const c_char) {
    space.delete_region(unsafe { CStr::from_ptr(name).to_str().unwrap() })
}

#[no_mangle]
extern "C" fn __ts_alloc_region(heap: *const Box<Rc<Heap>>, size: u64, align: u64, lazy: bool) -> *const Box<Rc<Region>> {
    match unsafe {
        if heap.is_null() {
            if lazy {
                GHEAP.lazy_alloc(size, align)
            } else {
                GHEAP.alloc(size, align)
            }
        } else {
            let p = heap.as_ref().unwrap();
            p.alloc(size, align)
        }
    } {
        Ok(region) => to_c_ptr(region),
        Err(msg) => panic!(msg)
    }
}

#[no_mangle]
extern "C" fn __ts_free_region(region: *const Box<Rc<Region>>) {
    std::mem::drop(unsafe { region.read() })
}

#[no_mangle]
extern "C" fn __ts_region_info(region: &Box<Rc<Region>>) -> *const MemInfo {
    Box::into_raw(Box::new(region.info))
}

#[no_mangle]
extern "C" fn __ts_heap(region: &Box<Rc<Region>>) -> *const Box<Rc<Heap>> {
    Box::into_raw(Box::new(Box::new(Heap::new(region.deref()))))
}

#[no_mangle]
extern "C" fn __ts_free_heap(heap: *const Box<Rc<Heap>>) {
    std::mem::drop(unsafe { heap.read() })
}

#[no_mangle]
extern "C" fn __ts_map_region(region: &Box<Rc<Region>>, base: u64) -> *const Box<Rc<Region>> {
    to_c_ptr(Region::remap(base, region.deref()))
}

#[no_mangle]
extern "C" fn __ts_map_region_partial(region: &Box<Rc<Region>>, base: u64, offset: u64, size: u64) -> *const Box<Rc<Region>> {
    to_c_ptr(Region::remap_partial(base, region.deref(), offset, size))
}

#[no_mangle]
extern "C" fn __ts_region_write_u8(region: &Box<Rc<Region>>, addr: u64, data: u8) {
    U8Access::write(region.deref().deref(), &addr, data)
}

#[no_mangle]
extern "C" fn __ts_region_write_u16(region: &Box<Rc<Region>>, addr: u64, data: u16) {
    U16Access::write(region.deref().deref(), &addr, data)
}

#[no_mangle]
extern "C" fn __ts_region_write_u32(region: &Box<Rc<Region>>, addr: u64, data: u32) {
    U32Access::write(region.deref().deref(), &addr, data)
}

#[no_mangle]
extern "C" fn __ts_region_write_u64(region: &Box<Rc<Region>>, addr: u64, data: u64) {
    U64Access::write(region.deref().deref(), &addr, data)
}

#[no_mangle]
extern "C" fn __ts_region_read_u8(region: &Box<Rc<Region>>, addr: u64) -> u8 {
    U8Access::read(region.deref().deref(), &addr)
}

#[no_mangle]
extern "C" fn __ts_region_read_u16(region: &Box<Rc<Region>>, addr: u64) -> u16 {
    U16Access::read(region.deref().deref(), &addr)
}

#[no_mangle]
extern "C" fn __ts_region_read_u32(region: &Box<Rc<Region>>, addr: u64) -> u32 {
    U32Access::read(region.deref().deref(), &addr)
}

#[no_mangle]
extern "C" fn __ts_region_read_u64(region: &Box<Rc<Region>>, addr: u64) -> u64 {
    U64Access::read(region.deref().deref(), &addr)
}


#[no_mangle]
extern "C" fn __ts_space_write_u8(space: &Rc<Space>, addr: u64, data: u8) {
    space.write_u8(&addr, data).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_write_u16(space: &Rc<Space>, addr: u64, data: u16) {
    space.write_u16(&addr, data).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_write_u32(space: &Rc<Space>, addr: u64, data: u32) {
    space.write_u32(&addr, data).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_write_u64(space: &Rc<Space>, addr: u64, data: u64) {
    space.write_u64(&addr, data).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_read_u8(space: &Rc<Space>, addr: u64) -> u8 {
    space.read_u8(&addr).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_read_u16(space: &Rc<Space>, addr: u64) -> u16 {
    space.read_u16(&addr).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_read_u32(space: &Rc<Space>, addr: u64) -> u32 {
    space.read_u32(&addr).unwrap()
}

#[no_mangle]
extern "C" fn __ts_space_read_u64(space: &Rc<Space>, addr: u64) -> u64 {
    space.read_u64(&addr).unwrap()
}


fn to_c_ptr(obj: Rc<Region>) -> *const Box<Rc<Region>> {
    Box::into_raw(Box::new(Box::new(obj)))
}
