# Rust Exploration
A repository to explore and experiment with intermediate Rust concepts and features.

## 1. Smart Pointers and Interior Mutability

### 1.1. Custom Implementation of `std::cell::Cell`
Exploring interior mutability by implementing a `Cell` struct with `get` and `set` methods:
- **`std::cell::UnsafeCell`:** The core primitive for interior mutability in Rust.
- **`unsafe` keyword:** Used to write code that ensures safety guarantees which the compiler cannot verify on its own.
- **`// SAFETY:` comments:** Documenting why the usage of `unsafe` is considered safe.
- **`std::marker::Sync` trait:** Indicates types that can be safely shared between threads.
- **`impl !Sync for UnsafeCell<T>`:** Implies that `UnsafeCell` is not thread-safe by default.

### 1.2. Custom Implementation of `std::cell::RefCell`
Learning about lifetimes and dynamic borrowing by implementing a `RefCell` struct with `borrow` and `borrow_mut` methods:
- **`Ref` struct:** A wrapper type for an immutably borrowed value from a `RefCell`.
- **`RefMut` struct:** A wrapper type for a mutably borrowed value from a `RefCell`.
- **`std::ops::Deref` and `DerefMut`:** Traits for overloading the `*` operator to dereference `Ref` and `RefMut`.
- **`std::ops::Drop` trait:** Allows customizing the behavior when a `Ref` or `RefMut` goes out of scope.

### 1.3. Custom Implementation of `std::rc::Rc`
Understanding single-threaded reference counting by implementing an `Rc` (Reference Counted) pointer:
- **`std::ptr::NonNull`:** A non-nullable pointer type used for manual memory management.
- **`std::marker::PhantomData`:** A zero-sized marker used to indicate that a struct owns data of a particular type.
- **Drop check:** Ensuring resources are properly freed when the reference count drops to zero.

### 1.4. Multi-Thread Safety with Smart Pointers
- **`Arc` (Atomic Reference Counting):** A thread-safe version of `Rc`.
- **`RwLock`:** A lock that allows multiple readers or a single writer.
- **`Mutex`:** A lock that ensures mutual exclusion for accessing shared data.

## 2. Declarative Macros
Exploring declarative macros by implementing a custom version of the `vec!` macro:
- **cargo-expand crate:** A tool for viewing the result of macro expansion and `#[derive]` expansion.
- **`#[macro_export]`:** An attribute that makes a macro available across crate boundaries.
- **Techniques for counting in `macro_rules!`:** Exploring patterns for counting and repetition in macros.

## 3. Iterators
Understanding iterators by implementing a custom version of `std::iter::Flatten` with `next` and `next_back` methods:
- **`std::iter::IntoIterator` trait:** A trait for converting a collection into an iterator.
- **`std::iter::Iterator` trait:** The core trait for iteration.
- **`DoubleEndedIterator` trait:** Allows iterating from both ends of a collection.
- **`IteratorExt` trait:** Adding additional iterator methods through an extension trait.

## 4. Channels
Learning about channels by implementing a custom multi-producer, single-consumer (MPSC) channel:
- **`Arc`:** Used to share ownership of data between `Sender` and `Receiver`.
- **`Mutex`:** Protects shared data and ensures safe access across threads.
- **`Condvar`:** A condition variable for signaling between threads, ensuring that a waiting thread is notified when data is available.

## 5. Atomics and Memory Ordering

In this section, I explore atomics and memory ordering by implementing a custom mutex and through various examples.

### 5.1. `.with_lock_v1()`

This implementation uses `.load()` and `.store()` on `AtomicBool` with `Ordering::Relaxed`.

#### Key Points:
- **Potential Race Condition:** Since both `.load()` and `.store()` use `Ordering::Relaxed`, there is no synchronization guarantee. This means that other threads can observe and act on stale or inconsistent data.
- **Simultaneous Access:** Two threads could simultaneously observe the `UNLOCKED` state in the `.load()` loop. As a result, they could both proceed to acquire the lock, leading to data races.
- **Unsafe Locking Mechanism:** The relaxed memory ordering in this implementation makes the locking mechanism unsafe, as it allows multiple threads to potentially enter the critical section simultaneously.

### 5.2. `.with_lock_v2()`

This implementation uses `.compare_exchange_weak()` on `AtomicBool` with `Ordering::Relaxed`.

#### Key Points:
- **CPU Load:** Using `.compare_exchange()` in a loop can create high CPU load due to constant attempts by threads to acquire the lock.
- **Partial Mitigation:** The use of `.compare_exchange_weak()` can partially mitigate this issue, as it allows for spurious failures. This means a thread may fail to acquire the lock even if the comparison succeeds, reducing contention slightly.
- **Memory Ordering Issues:** Despite these improvements, the use of `Ordering::Relaxed` still poses problems. Another thread may see an old value from a previously locked thread, leading to potential inconsistencies.

### 5.3. `.with_lock_v3()`

This implementation uses `.compare_exchange_weak()` on `AtomicBool` with `Ordering::Acquire` for locking and `Ordering::Release` for unlocking.

#### Key Points:
- **Correct Memory Ordering:** This implementation uses `Ordering::Acquire` when a thread acquires the lock and `Ordering::Release` when a thread releases it. This ensures proper synchronization, preventing other threads from observing inconsistent states.
- **Safe Locking Mechanism:** The use of acquire-release semantics provides the necessary guarantees that once a lock is acquired, all previous memory operations are visible, and once a lock is released, all subsequent memory operations by other threads will see the updated state.

### 5.4. Memory Ordering
The key memory orderings used are:

- **Relaxed:** No synchronization or ordering guarantees, only atomicity.
- **Acquire:** Synchronizes with `Release`. Ensures that subsequent reads and writes occur after the acquire operation.
- **Release:** Ensures that all previous reads and writes occur before the release operation. Synchronizes with `Acquire`.
- **SeqCst (Sequentially Consistent):** Provides the strongest ordering guarantees, enforcing a single global order of all operations.

### 5.5. Resources
- [Atomic Memory Ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
- [C++ std::memory_order](https://en.cppreference.com/w/cpp/atomic/memory_order)
- [CDSCHECKER: Checking Concurrent Data Structures Written with C/C++ Atomics](http://plrg.eecs.uci.edu/publications/c11modelcheck.pdf)
- [ThreadSanitizer](https://doc.rust-lang.org/stable/unstable-book/compiler-flags/sanitizer.html#threadsanitizer)
- [Loom](https://github.com/tokio-rs/loom)
