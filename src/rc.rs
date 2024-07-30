use crate::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;

struct Shared<T> {
    value: T,
    ref_count: Cell<usize>,
}

pub struct Rc<T> {
    shared: NonNull<Shared<T>>,
    _marker: PhantomData<Shared<T>>,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        let shared = Box::new(Shared {
            value,
            ref_count: Cell::new(1),
        });

        Rc {
            // SAFETY: Box does not give us a null pointer.
            shared: unsafe { NonNull::new_unchecked(Box::into_raw(shared)) },
            _marker: PhantomData,
        }
    }

    // Method to get the reference count for testing purposes.
    #[allow(dead_code)]
    fn ref_count(&self) -> usize {
        unsafe { self.shared.as_ref().ref_count.get() }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: `self.shared` is a Box that is only deallocated when the last Rc goes away.
        // The Box has not been deallocated, because this Rc is present. Deref is allowed.
        &unsafe { self.shared.as_ref() }.value
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        // SAFETY: `self.shared` is of type `NonNull<Shared<T>>`. The pointer is never null.
        let shared = unsafe { self.shared.as_ref() };
        let count = shared.ref_count.get();
        shared.ref_count.set(count + 1);
        Rc {
            shared: self.shared,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let shared = unsafe { self.shared.as_ref() };
        let count = shared.ref_count.get();
        match count {
            // SAFETY: This is the last Rc, and it is dropped. There will be no Rc, and no references to T.
            1 => drop(unsafe { Box::from_raw(self.shared.as_ptr()) }),
            // There are other Rcs, the Box will not be dropped.
            _ => shared.ref_count.set(count - 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rc_clone_twice() {
        let rc = Rc::new(String::from("Hello"));
        let rc_clone_1 = rc.clone();
        let rc_clone_2 = Rc::clone(&rc);
        assert_eq!(*rc, String::from("Hello"));
        assert_eq!(*rc_clone_1, String::from("Hello"));
        assert_eq!(*rc_clone_2, String::from("Hello"));
    }

    #[test]
    fn test_create_rc_clone_once_drop_rc_access_clone() {
        let rc = Rc::new(37);
        let rc_clone_1 = rc.clone();
        drop(rc);
        assert_eq!(*rc_clone_1, 37);
    }

    #[test]
    fn test_rc_with_refcell_create_and_clone_rc_modify_rc_check_clone() {
        let data = Rc::new(crate::refcell::RefCell::new(37));
        let data_clone = Rc::clone(&data);
        *data.borrow_mut().unwrap() = 73;
        assert_eq!(*data_clone.borrow().unwrap(), 73);
    }

    #[test]
    fn test_ref_count_check() {
        let rc = Rc::new(37);
        assert_eq!(rc.ref_count(), 1);
        let rc_clone_1 = rc.clone();
        assert_eq!(rc.ref_count(), 2);
        assert_eq!(rc_clone_1.ref_count(), 2);
        let rc_clone_2 = Rc::clone(&rc);
        assert_eq!(rc_clone_1.ref_count(), 3);
        drop(rc);
        assert_eq!(rc_clone_1.ref_count(), 2);
        drop(rc_clone_2);
        assert_eq!(rc_clone_1.ref_count(), 1);
    }
}
