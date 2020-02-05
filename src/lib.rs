mod list;

#[cfg(test)]
mod test;

//use std::rc::Rc;
//use crate::list::*;

#[derive(Copy, Clone, Debug)]
pub struct AllocationInfo {
    pub base: u64,
    pub size: u64,
}

//pub struct Allocator {
//    pub info: AllocationInfo,
//    list: Rc<List<AllocationInfo>>,
//}
//
//impl Allocator {
//    pub fn new(base: u64, size: u64) -> Allocator {
//        Allocator {
//            info: AllocationInfo { base: base, size: size },
//            list: Rc::new(List::Nil),
//        }
//    }
//}