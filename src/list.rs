use std::rc::Rc;
use std::ops::Deref;
use std::borrow::Borrow;
use crate::list::List::Cons;

pub enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}

impl<T> List<T> {
    pub fn cons(v: T, list: &Rc<Self>) -> Rc<Self> {
        Rc::new(Cons(v, Rc::clone(list)))
    }

    pub fn nil() -> Rc<Self> {
        Rc::new(List::Nil)
    }

    pub fn last<'a>(self:&'a Rc<Self>) -> &'a Rc<Self> {
        if let &List::Nil = self.cdr().as_ref() {
            self
        } else {
            self.cdr().last()
        }
    }

    pub fn car<'a>(self:&'a Rc<Self>) -> Option<&'a T> {
        if let List::Cons(v, _) = self.as_ref() {
            Some(v)
        } else {
            None
        }
    }

    pub fn cdr<'a>(self:&'a Rc<Self>) -> &'a Rc<Self> {
        if let List::Cons(_, list) = self.as_ref() {
            list
        } else {
            self
        }
    }
}

