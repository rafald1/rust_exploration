use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Cell {
            value: UnsafeCell::new(value),
        }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        // SAFETY: !Sync is implied because std::cell::UnsafeCell impl !Sync.
        // No other thread is modifying this value, since only this thread can mutate
        // and is executing get function instead.
        unsafe { *self.value.get() }
    }

    pub fn set(&self, value: T) {
        // SAFETY: !Sync is implied because std::cell::UnsafeCell impl !Sync.
        // No other thread is modifying this value, since only this thread can mutate.
        // By setting new value no references are invalidated, because no reference was ever given.
        unsafe { *self.value.get() = value }
    }
}
