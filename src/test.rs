use crate::list::*;
use crate::{AllocationInfo, Allocator};
use crate::LockedAllocator;
use std::sync::{mpsc, Arc};
use std::thread;
use std::sync::mpsc::Sender;
use std::borrow::Borrow;

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
fn list_delete() {
    let list1 = List::cons(AllocationInfo { base: 1, size: 1 },
                           &List::cons(AllocationInfo { base: 2, size: 2 }, &List::cons(AllocationInfo { base: 3, size: 3 }, &List::nil())));
    let (item, list2) = List::delete(&list1, |item| { item.car().unwrap().base == 2 });
    assert_eq!(item.unwrap().car().unwrap().size, 2);
    assert_eq!(list2.iter().count(), 2)
}

#[test]
fn basic_alloc() {
    let allocator = &mut Allocator::new(1, 9);
    assert_eq!(allocator.alloc(4, 1), Some(AllocationInfo { base: 1, size: 4 }));
    assert_eq!(allocator.alloc(2, 4), Some(AllocationInfo { base: 8, size: 2 }));
    assert_eq!(allocator.alloc(1, 1), Some(AllocationInfo { base: 5, size: 1 }));
    assert_eq!(allocator.alloc(2, 1), Some(AllocationInfo { base: 6, size: 2 }));
    assert_eq!(allocator.alloc(1, 1), None);
}

#[test]
fn basic_free() {
    let allocator = &mut Allocator::new(1, 9);
    assert_eq!(allocator.alloc(4, 1), Some(AllocationInfo { base: 1, size: 4 }));
    assert_eq!(allocator.alloc(2, 4), Some(AllocationInfo { base: 8, size: 2 }));
    assert_eq!(allocator.alloc(1, 1), Some(AllocationInfo { base: 5, size: 1 }));
    assert_eq!(allocator.alloc(2, 1), Some(AllocationInfo { base: 6, size: 2 }));
    assert_eq!(allocator.alloc(1, 1), None);
    allocator.free(8);
    allocator.free(5);
    allocator.free(1);
    allocator.free(6);
    assert!(allocator.free_blocks.iter().count() == 1);
    assert_eq!(allocator.free_blocks.car(), Some(AllocationInfo { base: 1, size: 9 }));
    assert_eq!(allocator.alloced_blocks.car(), None);
}

#[test]
fn basic_concurrency_alloc() {
    let allocator = Arc::new(LockedAllocator::new(1, 9));
    let (tx, rx) = mpsc::channel();
    fn do_job<F: Fn(&LockedAllocator) -> Option<AllocationInfo> + Send + 'static>(job: F, tx: &Sender<Option<AllocationInfo>>, la: &Arc<LockedAllocator>) {
        let done = mpsc::Sender::clone(tx);
        let _la = Arc::clone(&la);
        thread::spawn(move || {
            let result = job(_la.borrow());
//            println!("send, done!");
            done.send(result).unwrap();
        });
    };
    let mut addrs = vec![];

    do_job(|a| { a.alloc(4, 1) }, &tx, &allocator);
    do_job(|a| { a.alloc(2, 1) }, &tx, &allocator);
    do_job(|a| { a.alloc(1, 1) }, &tx, &allocator);
    do_job(|a| { a.alloc(2, 1) }, &tx, &allocator);
    for _ in 0..=3 {
        addrs.push(rx.recv().unwrap());
        println!("done!")
    }
    assert_eq!(allocator.alloc(1, 1), None);

    let mut handles = vec![];
    for block in addrs {
        let la = Arc::clone(&allocator);
        let handle = thread::spawn(move || {
            la.free(block.unwrap().base);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    {
        let inner = allocator.inner.lock().unwrap();
        assert!(inner.free_blocks.iter().count() == 1);
        assert_eq!(inner.free_blocks.car(), Some(AllocationInfo { base: 1, size: 9 }));
        assert_eq!(inner.alloced_blocks.car(), None);
    }
}