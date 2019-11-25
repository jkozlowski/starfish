use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

pub struct Shared<T> {
    v: Rc<RefCell<T>>,
}

impl<T> Shared<T> {
    pub fn new(t: T) -> Shared<T> {
        Shared {
            v: Rc::new(RefCell::new(t)),
        }
    }
}

impl<T: Sized> Clone for Shared<T> {
    #[inline]
    fn clone(&self) -> Shared<T> {
        Shared { v: self.v.clone() }
    }
}

impl<T> Shared<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        self.v.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.v.borrow_mut()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.v.as_ptr()
    }
}

impl<T: fmt::Display> fmt::Display for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl<T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}

impl<'a, T> Deref for Shared<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.as_ptr().as_ref().unwrap() }
    }
}

impl<'a, T> DerefMut for Shared<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_ptr().as_mut().unwrap() }
    }
}
