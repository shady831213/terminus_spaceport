use crate::list::List::{Cons, Nil};
use crate::MemInfo;
use std::borrow::Borrow;
use std::rc::Rc;

#[test]
fn list() {
    let list = Box::new(Cons(MemInfo { addr: 2, size: 0 },
                             Box::new(Cons(MemInfo { addr: 4, size: 0 },
                                           Box::new(Cons(MemInfo { addr: 0, size: 3 },
                                                         Box::new(Nil)))))));
    assert_eq!(list.cdr().car().unwrap().addr, 4);
    assert_eq!(list.cdr().cdr().car().unwrap().size, 3);
}