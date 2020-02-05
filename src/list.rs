use std::rc::Rc;

pub enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}

impl<T> List<T> {
    pub fn cons(v: T, list: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Cons(v, Rc::clone(list)))
    }

    pub fn nil() -> Rc<Self> {
        Rc::new(Self::Nil)
    }

    pub fn last<'a>(self: &'a Rc<Self>) -> &'a Rc<Self> {
        if let &Self::Nil = self.cdr().as_ref() {
            self
        } else {
            self.cdr().last()
        }
    }

    #[inline]
    pub fn iter<'a>(self: &'a Rc<Self>) -> Iter<'a, T> {
        Iter {
            cur:self
        }
    }

    pub fn car<'a>(self: &'a Rc<Self>) -> Option<&'a T> {
        if let Self::Cons(v, _) = self.as_ref() {
            Some(v)
        } else {
            None
        }
    }

    pub fn cdr<'a>(self: &'a Rc<Self>) -> &'a Rc<Self> {
        if let Self::Cons(_, list) = self.as_ref() {
            list
        } else {
            self
        }
    }
}

pub struct Iter<'a, T> {
    cur: &'a Rc<List<T>>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;
        self.cur = self.cur.cdr();
        if let List::Cons(v, _) = cur.as_ref() {
            Some(v)
        } else {
            None
        }
    }
}

