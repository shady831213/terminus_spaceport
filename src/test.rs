use crate::list::*;
use crate::AllocationInfo;

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
        assert_eq!(i.size, i.base);
        assert_eq!(i.base, id);
        id += 1;
    })
}