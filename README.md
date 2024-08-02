# Rust Exploration
A repository to explore and experiment with intermediate Rust concepts and features.

## Smart Pointers and Interior Mutability

### Custom Implementation of `std::cell::Cell`
Exploring interior mutability by implementing a `Cell` struct with `get` and `set` methods:
- **`std::cell::UnsafeCell`:** The core primitive for interior mutability in Rust.
- **`unsafe` keyword:** Used to write code that ensures safety guarantees which the compiler cannot verify on its own.
- **`// SAFETY:` comments:** Documenting why the usage of `unsafe` is considered safe.
- **`std::marker::Sync` trait:** Indicates types that can be safely shared between threads.
- **`impl !Sync for UnsafeCell<T>`:** Implies that `UnsafeCell` is not thread-safe by default.

### Custom Implementation of `std::cell::RefCell`
Learning about lifetimes and dynamic borrowing by implementing a `RefCell` struct with `borrow` and `borrow_mut` methods:
- **`Ref` struct:** A wrapper type for an immutably borrowed value from a `RefCell`.
- **`RefMut` struct:** A wrapper type for a mutably borrowed value from a `RefCell`.
- **`std::ops::Deref` and `DerefMut`:** Traits for overloading the `*` operator to dereference `Ref` and `RefMut`.
- **`std::ops::Drop` trait:** Allows customizing the behavior when a `Ref` or `RefMut` goes out of scope.

### Custom Implementation of `std::rc::Rc`
Understanding single-threaded reference counting by implementing an `Rc` (Reference Counted) pointer:
- **`std::ptr::NonNull`:** A non-nullable pointer type used for manual memory management.
- **`std::marker::PhantomData`:** A zero-sized marker used to indicate that a struct owns data of a particular type.
- **Drop check:** Ensuring resources are properly freed when the reference count drops to zero.

### Multi-Thread Safety with Smart Pointers
- **`Arc` (Atomic Reference Counting):** A thread-safe version of `Rc`.
- **`RwLock`:** A lock that allows multiple readers or a single writer.
- **`Mutex`:** A lock that ensures mutual exclusion for accessing shared data.

## Declarative Macros
Exploring declarative macros by implementing a custom version of the `vec!` macro:
- **cargo-expand crate:** A tool for viewing the result of macro expansion and `#[derive]` expansion.
- **`#[macro_export]`:** An attribute that makes a macro available across crate boundaries.
- **Techniques for counting in `macro_rules!`:** Exploring patterns for counting and repetition in macros.

## Iterators
Understanding iterators by implementing a custom version of `std::iter::Flatten` with `next` and `next_back` methods:
- **`std::iter::IntoIterator` trait:** A trait for converting a collection into an iterator.
- **`std::iter::Iterator` trait:** The core trait for iteration.
- **`DoubleEndedIterator` trait:** Allows iterating from both ends of a collection.
- **`IteratorExt` trait:** Adding additional iterator methods through an extension trait.

## Channels
Learning about channels by implementing a custom multi-producer, single-consumer (MPSC) channel:
- **`Arc`:** Used to share ownership of data between `Sender` and `Receiver`.
- **`Mutex`:** Protects shared data and ensures safe access across threads.
- **`Condvar`:** A condition variable for signaling between threads, ensuring that a waiting thread is notified when data is available.
