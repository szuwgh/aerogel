use crate::runtime::runtime;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::future::Future;
use std::iter::Sum;
const DEFAULT_CONCURRENCY: usize = 4;

pub trait IntoConcurrencyIterator: Sized {
    type Item: Send;
    type Iter: Iterator<Item = Self::Item> + Send;
    fn into_con_iter(self) -> ConIter<Self::Iter, runtime::GlobalRuntime>;
}

pub trait ConcurrencyIterator: Sized + Send {
    type It: Send;
    fn sum<R, S>(self) -> impl Future<Output = Result<S, runtime::RuntimeError>> + Send
    where
        S: Send + Sum<Self::It> + Sum<S>,
        Self::It: Send + 'static;
}

pub struct ConIter<I, R = runtime::GlobalRuntime> {
    iter: I,
    limit: usize,
    r: R,
}

impl<I, R> ConIter<I, R>
where
    I: Iterator + Send,
{
    pub fn buffered(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }

    pub fn runtime(mut self, r: R) -> Self {
        self.r = r;
        self
    }

    pub fn runtime_type<R2>(self, runtime: R2) -> Self {
        self
    }
}

pub struct SliceIter<'a, T: Send> {
    slice: &'a [T],
}

impl<'a, T: Send> Iterator for SliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (head, tail) = self.slice.split_first()?;
            self.slice = tail;
            Some(head)
        }
    }
}

pub struct Then<I, F, R = runtime::GlobalRuntime> {
    base: I,
    f: F,
    limit: usize,
    r: R,
}

impl<'a, T: Send + Sync> IntoConcurrencyIterator for &'a [T] {
    type Item = &'a T;
    type Iter = SliceIter<'a, T>;

    fn into_con_iter(self) -> ConIter<Self::Iter> {
        ConIter {
            iter: SliceIter { slice: self },
            limit: DEFAULT_CONCURRENCY,
            r: runtime::GlobalRuntime,
        }
    }
}

impl<'a, T: Send + Sync> IntoConcurrencyIterator for &'a Vec<T> {
    type Item = &'a T;
    type Iter = SliceIter<'a, T>;
    fn into_con_iter(self) -> ConIter<Self::Iter> {
        self.as_slice().into_con_iter()
    }
}

mod tests {
    use super::*;
    use futures::executor::block_on;
    async fn query_score(id: i32) -> i32 {
        return id + 1;
    }
    #[test]
    fn test_sum() {
        // let a = vec![1, 2, 3, 4, 5, 6];
        // let res = block_on(async {
        //     (&a).into_con_iter()
        //         .then(|id| async move { query_score(*id).await })
        //         .sum::<i32>()
        //         .await
        // });
        // println!("res:{}", res);
    }
}
