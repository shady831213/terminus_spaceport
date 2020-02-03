mod list;

use list::List::{Cons, Nil};
use list::MemInfo;
use std::borrow::Borrow;
use crate::list::List;
use std::rc::Rc;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn list() {
    let list = Box::new(Cons(MemInfo { addr: 2, size: 0 },
                             Box::new(Cons(MemInfo { addr: 4, size: 0 },
                                           Box::new(Cons(MemInfo { addr: 0, size: 3 },
                                                         Box::new(Nil)))))));
    assert_eq!(list.cdr().car().unwrap().addr, 4);
    assert_eq!(list.cdr().cdr().car().unwrap().size, 3);
}
