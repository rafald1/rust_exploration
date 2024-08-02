use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
}

struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        drop(inner);
        self.shared.available.notify_one();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);
        Sender {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner);

        if was_last {
            self.shared.available.notify_one();
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}

impl<T> Receiver<T> {
    pub fn receive(&mut self) -> Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t);
        }

        let mut inner = self.shared.inner.lock().unwrap();

        loop {
            match inner.queue.pop_front() {
                Some(t) => {
                    std::mem::swap(&mut self.buffer, &mut inner.queue);
                    return Some(t);
                }
                None if inner.senders == 0 => return None,
                None => {
                    inner = self.shared.available.wait(inner).unwrap();
                }
            }
        }
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.receive()
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::new(),
        senders: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        available: Condvar::new(),
    };
    let shared = Arc::new(shared);
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone(),
            buffer: Default::default(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_send_and_receive() {
        let (mut tx, mut rx) = channel();
        tx.send(37);
        assert_eq!(rx.receive(), Some(37));
    }

    #[test]
    fn test_multiple_send_receive() {
        let (mut tx, mut rx) = channel();
        tx.send(1);
        tx.send(2);
        assert_eq!(rx.receive(), Some(1));
        assert_eq!(rx.receive(), Some(2));
    }

    #[test]
    fn test_closed_tx() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.receive(), None);
    }

    #[test]
    fn test_closed_rx() {
        let (mut tx, rx) = channel();
        drop(rx);
        tx.send(42);
    }

    #[test]
    fn test_concurrent_send_receive() {
        let (mut tx, mut rx) = channel();
        let mut handles = Vec::new();

        for i in 0..10 {
            let mut tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                tx_clone.send(i);
            });
            handles.push(handle);
        }

        tx.send(10);

        for handle in handles {
            handle.join().unwrap();
        }

        let mut received_values = Vec::new();
        (0..11).for_each(|_| received_values.push(rx.receive().unwrap()));
        received_values.sort_unstable();
        assert_eq!(received_values, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_receive_blocks_until_send() {
        let (mut tx, mut rx) = channel();
        let handle = thread::spawn(move || {
            assert_eq!(rx.receive(), Some(37));
        });
        thread::sleep(std::time::Duration::from_millis(100));
        tx.send(37);
        handle.join().unwrap();
    }
}
