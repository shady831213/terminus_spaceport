use std::rc::Rc;

pub enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}

impl<T> List<T> where T: Copy {
    pub fn cons(v: T, list: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Cons(v, Rc::clone(list)))
    }

    pub fn nil() -> Rc<Self> {
        Rc::new(Self::Nil)
    }

    pub fn append(list1: &Rc<Self>, list2: &Rc<Self>) -> Rc<Self> {
        match list1.as_ref() {
            Self::Nil => Rc::clone(list2),
            Self::Cons(v, l) => Self::cons(*v, &Self::append(l, list2))
        }
    }

    pub fn last<'a>(self: &'a Rc<Self>) -> &'a Rc<Self> {
        if let &Self::Nil = self.cdr().as_ref() {
            self
        } else {
            self.cdr().last()
        }
    }

    pub fn iter<'a>(self: &'a Rc<Self>) -> Iter<'a, T> {
        Iter {
            cur: self
        }
    }

    pub fn car(self: &Rc<Self>) -> Option<T> {
        if let Self::Cons(v, _) = self.as_ref() {
            Some(*v)
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

impl<'a, T> Iterator for Iter<'a, T> where T: Copy {
    type Item = &'a Rc<List<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;
        self.cur = self.cur.cdr();
        if let &List::Nil = cur.as_ref() {
            None
        } else {
            Some(cur)
        }
    }
}

