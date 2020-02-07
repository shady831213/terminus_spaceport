use std::sync::Arc;

pub enum List<T> {
    Cons(T, Arc<List<T>>),
    Nil,
}

impl<T> List<T> where T: Copy {
    pub fn cons(v: T, list: &Arc<Self>) -> Arc<Self> {
        Arc::new(Self::Cons(v, Arc::clone(list)))
    }

    pub fn nil() -> Arc<Self> {
        Arc::new(Self::Nil)
    }

    pub fn append(list1: &Arc<Self>, list2: &Arc<Self>) -> Arc<Self> {
        match list1.as_ref() {
            Self::Nil => Arc::clone(list2),
            Self::Cons(v, l) => Self::cons(*v, &Self::append(l, list2))
        }
    }

    pub fn last<'a>(self: &'a Arc<Self>) -> &'a Arc<Self> {
        if let &Self::Nil = self.cdr().as_ref() {
            self
        } else {
            self.cdr().last()
        }
    }

    pub fn iter<'a>(self: &'a Arc<Self>) -> Iter<'a, T> {
        Iter {
            cur: self
        }
    }

    pub fn car(self: &Arc<Self>) -> Option<T> {
        if let Self::Cons(v, _) = self.as_ref() {
            Some(*v)
        } else {
            None
        }
    }

    pub fn cdr<'a>(self: &'a Arc<Self>) -> &'a Arc<Self> {
        if let Self::Cons(_, list) = self.as_ref() {
            list
        } else {
            self
        }
    }
}

pub struct Iter<'a, T> {
    cur: &'a Arc<List<T>>
}

impl<'a, T> Iterator for Iter<'a, T> where T: Copy {
    type Item = &'a Arc<List<T>>;
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

