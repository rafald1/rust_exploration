pub fn flatten<I>(iter: I) -> Flatten<I::IntoIter>
where
    I: IntoIterator,
    I::Item: IntoIterator,
{
    Flatten::new(iter.into_iter())
}

pub struct Flatten<I>
where
    I: Iterator,
    I::Item: IntoIterator,
{
    outer: I,
    front_iter: Option<<I::Item as IntoIterator>::IntoIter>,
    back_iter: Option<<I::Item as IntoIterator>::IntoIter>,
}

impl<I> Flatten<I>
where
    I: Iterator,
    I::Item: IntoIterator,
{
    fn new(iter: I) -> Self {
        Flatten {
            outer: iter,
            front_iter: None,
            back_iter: None,
        }
    }
}

impl<I> Iterator for Flatten<I>
where
    I: Iterator,
    I::Item: IntoIterator,
{
    type Item = <I::Item as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(front_iter) = self.front_iter.as_mut() {
                if let Some(item) = front_iter.next() {
                    return Some(item);
                }
                self.front_iter = None;
            }

            if let Some(next_inner) = self.outer.next() {
                self.front_iter = Some(next_inner.into_iter());
            } else {
                return self.back_iter.as_mut()?.next();
            }
        }
    }
}

impl<I> DoubleEndedIterator for Flatten<I>
where
    I: DoubleEndedIterator,
    I::Item: IntoIterator,
    <I::Item as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(back_iter) = self.back_iter.as_mut() {
                if let Some(item) = back_iter.next_back() {
                    return Some(item);
                }
                self.back_iter = None;
            }

            if let Some(next_back_inner) = self.outer.next_back() {
                self.back_iter = Some(next_back_inner.into_iter());
            } else {
                return self.front_iter.as_mut()?.next_back();
            }
        }
    }
}

pub trait IteratorExt: Iterator + Sized {
    fn our_flatten(self) -> Flatten<Self>
    where
        Self::Item: IntoIterator;
}

impl<T> IteratorExt for T
where
    T: Iterator,
{
    fn our_flatten(self) -> Flatten<Self>
    where
        Self::Item: IntoIterator,
    {
        flatten(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_empty() {
        assert!(flatten::<Vec<Vec<()>>>(vec![vec![], vec![]])
            .next()
            .is_none());
    }

    #[test]
    fn test_element_count() {
        assert_eq!(flatten::<Vec<Vec<()>>>(vec![vec![], vec![]]).count(), 0);
        assert_eq!(flatten(vec![vec![37]]).count(), 1);
        assert_eq!(flatten(vec![vec![37, 73]]).count(), 2);
        assert_eq!(flatten(vec![vec![37], vec![73]]).count(), 2);
        assert_eq!(flatten(vec![vec![37, 73, 137], vec![], vec![0]]).count(), 4);
    }

    #[test]
    fn test_reverse() {
        assert_eq!(
            flatten(std::iter::once(vec![0, 1, 2]))
                .rev()
                .collect::<Vec<_>>(),
            vec![2, 1, 0]
        );
        assert_eq!(
            flatten(vec![vec![0], vec![1], vec![2]])
                .rev()
                .collect::<Vec<_>>(),
            vec![2, 1, 0]
        );
    }

    #[test]
    fn test_inf_iterator() {
        let mut iter = flatten((1..).map(|i| i..=2 * i));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(3));
    }

    #[test]
    fn test_next_and_next_back_together() {
        let mut iter = flatten(vec![vec![1, 2, 3], vec![4, 5]]);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(5));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_flatten_twice() {
        assert_eq!(flatten(flatten(vec![vec![vec![0, 1], vec![2]]])).count(), 3);
    }

    #[test]
    fn test_our_flatten_method() {
        assert_eq!(vec![vec![0, 1]].into_iter().our_flatten().count(), 2);
    }

    #[test]
    fn test_flatten_complex() {
        let result: Vec<_> = vec![vec![vec![1, 2], vec![3]], vec![vec![4], vec![5, 6]]]
            .into_iter()
            .our_flatten()
            .our_flatten()
            .collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }
}
