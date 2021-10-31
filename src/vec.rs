// Simple statically allocated vec.
#[derive(Clone)]
pub struct Vec<T, const C: usize> {
    data: [Option<T>; C],
    len: usize,
}

impl<T: Copy, const C: usize> Default for Vec<T, C> {
    fn default() -> Self {
        Self {
            data: [Option::default(); C],
            len: 0,
        }
    }
}

impl<T: core::fmt::Debug, const C: usize> core::fmt::Debug for Vec<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Vec")
            .field("data", &self.data)
            .field("len", &self.len)
            .finish()
    }
}

impl<T: Copy, const C: usize> Vec<T, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item: T) -> Option<T> {
        match self.data.get_mut(self.len) {
            Some(loc) => {
                self.len += 1;
                *loc = Some(item);
                None
            }
            None => Some(item),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            match self.data.get_mut(self.len - 1) {
                Some(loc) => {
                    self.len -= 1;
                    core::mem::take(loc)
                }
                None => None,
            }
        } else {
            None
        }
    }

    pub fn first(&self) -> Option<&T> {
        match self.data.get(0) {
            Some(loc) => loc.as_ref(),
            None => None,
        }
    }

    pub fn last(&self) -> Option<&T> {
        match self.data.get(self.len - 1) {
            Some(loc) => loc.as_ref(),
            None => None,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        C
    }
}

impl<T, const C: usize> IntoIterator for Vec<T, C> {
    type Item = T;

    type IntoIter = VecIter<T, C>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            data: self.data,
            head: 0,
            tail: self.len - 1,
        }
    }
}

pub struct VecIter<T, const C: usize> {
    data: [Option<T>; C],
    head: usize,
    tail: usize,
}

impl<T, const C: usize> Iterator for VecIter<T, C> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.get_mut(self.head) {
            Some(loc) => {
                self.head += 1;
                core::mem::take(loc)
            }
            None => None,
        }
    }
}

impl<T, const C: usize> DoubleEndedIterator for VecIter<T, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.data.get_mut(self.tail) {
            Some(loc) => {
                if self.tail > 0 {
                    self.tail -= 1
                };
                core::mem::take(loc)
            }
            None => None,
        }
    }
}

#[test]
fn new() {
    let vec: Vec<i32, 16> = Vec::new();
    assert_eq!(vec.capacity(), 16);
}

#[test]
fn push() {
    let mut vec: Vec<i32, 4> = Vec::new();

    assert_eq!(vec.push(34), None);
    assert_eq!(vec.data, [Some(34), None, None, None]);

    assert_eq!(vec.push(-12), None);
    assert_eq!(vec.data, [Some(34), Some(-12), None, None]);

    assert_eq!(vec.push(-33), None);
    assert_eq!(vec.data, [Some(34), Some(-12), Some(-33), None]);

    assert_eq!(vec.push(23), None);
    assert_eq!(vec.data, [Some(34), Some(-12), Some(-33), Some(23)]);

    assert_eq!(vec.push(11), Some(11));
    assert_eq!(vec.data, [Some(34), Some(-12), Some(-33), Some(23)]);
}

#[test]
fn pop() {
    let mut vec: Vec<i32, 4> = Vec::new();
    vec.push(11);
    vec.push(22);
    vec.push(33);

    assert_eq!(vec.pop(), Some(33));
    assert_eq!(vec.pop(), Some(22));
    assert_eq!(vec.pop(), Some(11));
    assert_eq!(vec.pop(), None);
}

#[test]
fn into_iter() {
    let mut vec: Vec<i32, 4> = Vec::new();
    vec.push(11);
    vec.push(22);
    vec.push(33);

    let mut iter = vec.into_iter();
    assert_eq!(iter.next(), Some(11));
    assert_eq!(iter.next_back(), Some(33));
    assert_eq!(iter.next(), Some(22));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}
