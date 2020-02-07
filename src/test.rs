use crate::list::*;
use crate::AllocationInfo;
use crate::Allocator;

#[test]
fn list_basic() {
    let list = List::cons(AllocationInfo { base: 2, size: 0 },
                          &List::cons(AllocationInfo { base: 4, size: 0 }, &List::cons(AllocationInfo { base: 0, size: 3 }, &List::nil())));
    assert_eq!(list.cdr().car().unwrap().base, 4);
    assert_eq!(list.cdr().cdr().car().unwrap().size, 3);

    let append = List::cons(AllocationInfo { base: 7, size: 0 }, &List::cons(AllocationInfo { base: 5, size: 6 }, &list));
    assert_eq!(append.car().unwrap().base, 7);
    assert_eq!(append.cdr().car().unwrap().base, 5);
    assert_eq!(append.cdr().cdr().car().unwrap().base, 2);
    assert_eq!(append.last().car().unwrap().size, 3);
}

#[test]
fn list_iter() {
    let list = List::cons(AllocationInfo { base: 1, size: 1 },
                          &List::cons(AllocationInfo { base: 2, size: 2 }, &List::cons(AllocationInfo { base: 3, size: 3 }, &List::nil())));
    let mut id: u64 = 1;
    list.iter().for_each(|i| {
        assert_eq!(i.car().unwrap().size, i.car().unwrap().base);
        assert_eq!(i.car().unwrap().base, id);
        id += 1;
    });
    assert_eq!(list.iter().count(), 3)
}

#[test]
fn list_append() {
    let list1 = List::cons(AllocationInfo { base: 1, size: 1 },
                           &List::cons(AllocationInfo { base: 2, size: 2 }, &List::cons(AllocationInfo { base: 3, size: 3 }, &List::nil())));
    let list2 = List::cons(AllocationInfo { base: 4, size: 4 },
                           &List::cons(AllocationInfo { base: 5, size: 5 }, &List::cons(AllocationInfo { base: 6, size: 6 }, &List::nil())));
    let mut id: u64 = 1;
    List::append(&list1, &list2).iter().for_each(|i| {
        assert_eq!(i.car().unwrap().size, i.car().unwrap().base);
        assert_eq!(i.car().unwrap().base, id);
        id += 1;
    });
    assert_eq!(List::append(&list1, &list2).iter().count(), 6)
}

#[test]
fn basic_alloc() {
    let allocator = &Allocator::new(1, 9);
    assert_eq!(allocator.alloc(4, 1), Some(AllocationInfo{base:1, size:4}));
    assert_eq!(allocator.alloc(2, 4), Some(AllocationInfo{base:8, size:2}));
    assert_eq!(allocator.alloc(1, 1), Some(AllocationInfo{base:5, size:1}));
    assert_eq!(allocator.alloc(2, 1), Some(AllocationInfo{base:6, size:2}));
    assert_eq!(allocator.alloc(1, 1), None);
}