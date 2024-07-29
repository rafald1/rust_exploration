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

#[cfg(test)]
mod tests {
    use super::Cell;

    #[test]
    fn test_new() {
        let cell = Cell::new(37);
        assert_eq!(cell.get(), 37);
    }

    #[test]
    fn test_set() {
        let cell = Cell::new(37);
        cell.set(73);
        assert_eq!(cell.get(), 73);
    }

    #[test]
    fn test_set_multiple_times() {
        let cell = Cell::new(37);
        cell.set(73);
        cell.set(137);
        assert_eq!(cell.get(), 137);
    }

    #[test]
    fn test_set_and_get_with_different_types() {
        let cell = Cell::new("hello");
        assert_eq!(cell.get(), "hello");
        cell.set("world");
        assert_eq!(cell.get(), "world");
    }

    #[test]
    fn test_copy_types() {
        let cell = Cell::new((37, 73, 137));
        assert_eq!(cell.get(), (37, 73, 137));
        cell.set((0, 0, 0));
        assert_eq!(cell.get(), (0, 0, 0));
    }
}
