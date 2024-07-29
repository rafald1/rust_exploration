use crate::cell::Cell;
use std::cell::UnsafeCell;

/// The state of a `RefCell`, tracking how it is being accessed.
#[derive(Copy, Clone, Debug, PartialEq)]
enum RefCellState {
    Shared(usize),
    Exclusive,
}

/// A cell providing interior mutability with dynamic borrowing.
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<RefCellState>,
}

/// An immutable reference to the value inside a `RefCell`.
pub struct Ref<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<T> std::ops::Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: A `Ref` is only created if no exclusive references exist.
        // State is set to Shared and no exclusive reference will be given out.
        // Dereferencing into a shared reference is safe.
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefCellState::Exclusive | RefCellState::Shared(0) => unreachable!(),
            RefCellState::Shared(count) => self.refcell.state.set(RefCellState::Shared(count - 1)),
        }
    }
}

/// A mutable reference to the value inside a `RefCell`.
pub struct RefMut<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<T> std::ops::Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: see safety for DerefMut
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> std::ops::DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: A `RefMut` is only created if no other references exist.
        // State is set to Exclusive and no future references are given out.
        // An exclusive lease has been acquired on the inner value and mutably dereferencing is allowed.
        unsafe { &mut *self.refcell.value.get() }
    }
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefCellState::Shared(_) => unreachable!(),
            RefCellState::Exclusive => self.refcell.state.set(RefCellState::Shared(0)),
        }
    }
}

impl<T> RefCell<T> {
    /// Creates a new `RefCell` containing `value`.
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(RefCellState::Shared(0)),
        }
    }

    /// Attempts to borrow the value immutably.
    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        match self.state.get() {
            RefCellState::Shared(count) => {
                self.state.set(RefCellState::Shared(count + 1));
                Some(Ref { refcell: self })
            }
            RefCellState::Exclusive => None,
        }
    }

    /// Attempts to borrow the value mutably.
    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        match self.state.get() {
            RefCellState::Shared(0) => {
                // SAFETY: no other references are currently given, because state is Shared(0).
                self.state.set(RefCellState::Exclusive);
                Some(RefMut { refcell: self })
            }
            RefCellState::Shared(_) | RefCellState::Exclusive => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_twice() {
        let data = RefCell::new(37);

        let observer_1 = data.borrow();
        let observer_2 = data.borrow();
        assert_eq!(data.state.get(), RefCellState::Shared(2));
        assert_eq!(*observer_1.unwrap(), 37);
        assert_eq!(*observer_2.unwrap(), 37);
    }

    #[test]
    fn test_borrow_mut_once() {
        let data = RefCell::new(37);

        let mut modifier = data.borrow_mut().unwrap();
        assert_eq!(*modifier, 37);
        *modifier = 73;
        assert_eq!(data.state.get(), RefCellState::Exclusive);
    }

    #[test]
    fn test_borrow_once_drop_borrow_mut_once_modify_drop_borrow_once() {
        let data = RefCell::new(37);

        {
            let observer = data.borrow();
            assert_eq!(*observer.unwrap(), 37);
        }
        {
            let mut modifier = data.borrow_mut().unwrap();
            assert_eq!(*modifier, 37);
            *modifier = 73;
        }
        let observer = data.borrow();
        assert_eq!(*observer.unwrap(), 73);
    }

    #[test]
    #[should_panic]
    fn test_borrow_once_and_borrow_mut_once() {
        let data = RefCell::new(37);

        let observer = data.borrow();
        let modifier = data.borrow_mut();
        assert_eq!(*observer.unwrap(), 37);
        let _ = *modifier.unwrap();
    }
}
