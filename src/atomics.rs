use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Mutex<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            value: UnsafeCell::new(t),
        }
    }

    pub fn with_lock_v1<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.locked.load(Ordering::Relaxed) != UNLOCKED {
            std::hint::spin_loop();
        }
        std::thread::yield_now();
        self.locked.store(LOCKED, Ordering::Relaxed);
        // SAFETY: this thread holds the lock, therefore we can create a mutable reference
        let ret = f(unsafe { &mut *self.value.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret
    }

    pub fn with_lock_v2<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // MESI protocol: stay in Shared state when locked
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::hint::spin_loop();
                std::thread::yield_now();
            }
            std::thread::yield_now();
        }
        // SAFETY: this thread holds the lock, therefore we can create a mutable reference
        let ret = f(unsafe { &mut *self.value.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret
    }

    pub fn with_lock_v3<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // MESI protocol: stay in Shared state when locked
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::hint::spin_loop();
            }
        }
        // SAFETY: this thread holds the lock, therefore we can create a mutable reference
        let ret = f(unsafe { &mut *self.value.get() });
        self.locked.store(UNLOCKED, Ordering::Release);
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::spawn;

    #[test]
    fn test_computed_value_is_different_than_expected_for_load_and_store_version() {
        let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));
        let handles: Vec<_> = (0..100)
            .map(|_| {
                spawn(move || {
                    for _ in 0..1000 {
                        l.with_lock_v1(|v| {
                            *v += 1;
                        })
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        // Should be not equal confirming our with_lock_v1 isn't safe
        assert_ne!(l.with_lock_v1(|v| *v), 100 * 1000)
    }

    #[test]
    fn test_exchange_weak_version_with_relaxed_ordering() {
        let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));
        let handles: Vec<_> = (0..100)
            .map(|_| {
                spawn(move || {
                    for _ in 0..1000 {
                        l.with_lock_v2(|v| {
                            *v += 1;
                        })
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        // This test can sometimes fail, confirming our with_lock_v2 isn't safe
        assert_eq!(l.with_lock_v2(|v| *v), 100 * 1000)
    }

    #[test]
    fn test_relaxed_ordering_example() {
        use std::sync::atomic::AtomicUsize;
        let x: &'static AtomicUsize = Box::leak(Box::new(AtomicUsize::new(0)));
        let y: &'static AtomicUsize = Box::leak(Box::new(AtomicUsize::new(0)));
        let thread_1 = spawn(move || {
            let read_1 = y.load(Ordering::Relaxed); // A
            x.store(read_1, Ordering::Relaxed); // B
            read_1
        });
        let thread_2 = spawn(move || {
            let read_2 = x.load(Ordering::Relaxed); // C
            y.store(37, Ordering::Relaxed); // D
            read_2
        });
        let _read_1 = thread_1.join().unwrap();
        let _read_2 = thread_2.join().unwrap();
        // Atomic operations tagged Ordering::Relaxed are not synchronization operations;
        // They do not impose an order among concurrent memory accesses. They only guarantee
        // atomicity and modification order consistency.
        // This code is allowed to produce read_1 == read_2 == 37 because,
        // although A is sequenced-before B within thread 1 and C is sequenced before D within thread 2,
        // nothing prevents D from appearing before A in the modification order of y,
        // and B from appearing before C in the modification order of x.
        // The side effect of D on y could be visible to the load A in thread 1 while
        // the side effect of B on x could be visible to the load C in thread 2.
        // In particular, this may occur if D is completed before C in thread 2,
        // either due to compiler reordering or at runtime.
    }

    #[test]
    fn test_exchange_weak_version_with_proper_memory_ordering() {
        let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));
        let handles: Vec<_> = (0..100)
            .map(|_| {
                spawn(move || {
                    for _ in 0..1000 {
                        l.with_lock_v3(|v| {
                            *v += 1;
                        })
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(l.with_lock_v3(|v| *v), 100 * 1000)
    }

    #[test]
    fn test_acquire_release_ordering_example() {
        use std::sync::atomic::AtomicUsize;
        let x: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
        let y: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
        let z: &'static AtomicUsize = Box::leak(Box::new(AtomicUsize::new(0)));

        let _write_x = spawn(move || x.store(true, Ordering::Release));
        let _write_y = spawn(move || y.store(true, Ordering::Release));
        let read_x_then_y = spawn(move || {
            while !x.load(Ordering::Acquire) {}
            if y.load(Ordering::Acquire) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        });
        let read_y_then_x = spawn(move || {
            while !y.load(Ordering::Acquire) {}
            if x.load(Ordering::Acquire) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        });
        read_x_then_y.join().unwrap();
        read_y_then_x.join().unwrap();
        let z = z.load(Ordering::SeqCst);
        // Possible values for z:
        // z == 2: write_x -> write_y -> read_x_then_y -> read_y_then_x
        // z == 1: write_x -> read_x_then_y -> write_y -> read_y_then_x
        assert_ne!(z, 0)
        // z == 0:
        // This example demonstrates a situation where sequential ordering is necessary.
        // Any other ordering may trigger the assert because it would be possible for
        // the threads read_x_then_y and read_y_then_x to observe changes to the atomics x and y
        // in opposite order.
        // Total sequential ordering requires a full memory fence CPU instruction
        // on all multicore systems. This may become a performance bottleneck since it forces
        // the affected memory accesses to propagate to every core.
    }

    #[test]
    fn test_seqcst_ordering_example() {
        use std::sync::atomic::AtomicUsize;
        let x: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
        let y: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
        let z: &'static AtomicUsize = Box::leak(Box::new(AtomicUsize::new(0)));

        let _write_x = spawn(move || x.store(true, Ordering::SeqCst));
        let _write_y = spawn(move || y.store(true, Ordering::SeqCst));
        let read_x_then_y = spawn(move || {
            while !x.load(Ordering::SeqCst) {}
            if y.load(Ordering::SeqCst) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        });
        let read_y_then_x = spawn(move || {
            while !y.load(Ordering::SeqCst) {}
            if x.load(Ordering::SeqCst) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        });
        read_x_then_y.join().unwrap();
        read_y_then_x.join().unwrap();
        let z = z.load(Ordering::SeqCst);
        assert_ne!(z, 0)
        // z == 0 is no longer possible.
    }
}
