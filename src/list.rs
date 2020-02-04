use std::rc::Rc;
use std::ops::Deref;
use std::borrow::Borrow;

pub enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    pub fn car(&self) -> Option<&T> {
        if let List::Cons(v, _) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn cdr(&self) -> &List<T> {
        if let List::Cons(_, list) = self {
            list
        } else {
            &List::Nil
        }
    }
}


