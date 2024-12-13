use crate::get_num;
use crate::join;
use crate::Runtime;
use crate::RUNTIME;
use std::iter;
use std::iter::Sum;
use std::marker::PhantomData;
pub struct Map<I, F> {
    pub(crate) iter: I,
    f: F,
}

impl<I, F: Sync + Send, R: Send + Zero> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    F: FnMut(I::It) -> R,
{
    type It = R;
    fn divide<C: Consumer<Self::It>>(self, c: C) -> C::Result {
        todo!()
    }
}
unsafe impl<S: Send> Send for SumConsumer<S> {}
pub trait IntoParallelIterator {
    type Iter: ParallelIterator;
    fn into_aer_iter(&self) -> Self::Iter;
}

fn sum<PI, S>(iter: PI) -> S
where
    PI: ParallelIterator,
    S: Sum<PI::It> + Sum + Send,
{
    iter.divide(SumConsumer::new())
}

pub trait Zero {
    fn zero() -> Self;
}

#[inline]
fn add<T: Sum>(left: T, right: T) -> T {
    [left, right].into_iter().sum()
}

trait Folder<T> {
    type Result;
    fn consume(self, item: T) -> Self;

    fn result(self) -> Self::Result;
}

trait Reducer<T> {
    fn reduce(&self, left: T, right: T) -> T;
}

impl<S, T> Folder<T> for SumFolder<S>
where
    S: Sum<T> + Sum + Send,
{
    type Result = S;
    #[inline]
    fn consume(mut self, item: T) -> Self {
        let item = self.reducer.reduce(self.s, iter::once(item).sum()); //add(self.s, iter::once(item).sum()); //;
        self.s = item;
        self
    }

    #[inline]
    fn result(self) -> Self::Result {
        self.s
    }
}

struct SumFolder<S> {
    reducer: SumReducer<S>,
    s: S,
}

struct SumReducer<S>(PhantomData<*const S>);

trait Consumer<T>: Send + Sized {
    type Folder: Folder<T, Result = Self::Result>;
    type Reducer: Reducer<Self::Result>;
    type Result: Send;
    fn folder(&self) -> Self::Folder;
    fn reducer(&self) -> Self::Reducer;
}

impl<S, T> Consumer<T> for SumConsumer<S>
where
    S: Send + Sum + Sum<T>,
{
    type Folder = SumFolder<S>;
    type Reducer = SumReducer<S>;
    type Result = S;
    #[inline]
    fn folder(&self) -> Self::Folder {
        SumFolder {
            reducer: self.to_reducer(),
            s: iter::empty::<T>().sum(),
        }
    }

    #[inline]
    fn reducer(&self) -> Self::Reducer {
        SumReducer::new()
    }
}

impl<S: Sum> Reducer<S> for SumReducer<S> {
    #[inline]
    fn reduce(&self, left: S, right: S) -> S {
        add(left, right)
    }
}

impl<S: Send + Sum> SumReducer<S> {
    fn new() -> SumReducer<S> {
        SumReducer(PhantomData)
    }
}

struct SumConsumer<S>(PhantomData<*const S>);

impl<S: Send + Sum> SumConsumer<S> {
    fn new() -> SumConsumer<S> {
        SumConsumer(PhantomData)
    }
    fn to_reducer(&self) -> SumReducer<S> {
        self.reducer()
    }
}

pub trait ParallelIterator: Sized + Send {
    type It: Send;
    fn sum<S>(self) -> S
    where
        S: Send + Sum<Self::It> + Sum<S>,
    {
        sum(self)
    }

    fn map<F, R>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::It) -> R,
    {
        Map { iter: self, f: f }
    }

    fn divide<C: Consumer<Self::It>>(self, c: C) -> C::Result;
}

pub struct SliceIter<'a, T: Send> {
    slice: &'a [T],
}

impl<'a, T: Send> SliceIter<'a, T> {
    fn len(&self) -> usize {
        self.slice.len()
    }
}

impl<'a, T: Send + Sync> ParallelIterator for SliceIter<'a, T> {
    type It = &'a T;

    fn divide<C: Consumer<Self::It>>(self, c: C) -> C::Result {
        divide(self.len(), get_num(), self, c)
    }
}

#[inline]
fn divide<P, C>(len: usize, cost: usize, p: P, c: C) -> C::Result
where
    P: Producer,
    C: Consumer<P::It>,
{
    divide_job(len, cost, p, &c)
}

fn divide_job<P, C>(len: usize, cost: usize, p: P, c: &C) -> C::Result
where
    P: Producer,
    C: Consumer<P::It>,
{
    if cost > 1 && len > 1 {
        let mid = len / 2;
        let (left_producer, right_producer) = p.split_at(mid);
        let (left_result, right_result) = join(
            || divide_job(mid, cost / 2, left_producer, c),
            || divide_job(len - mid, cost / 2, right_producer, c),
        );
        let reducer = c.reducer();
        reducer.reduce(left_result, right_result)
    } else {
        p.produce(c)
    }
}

impl<'a, T: 'a + Send> IntoIterator for SliceIter<'a, T> {
    type Item = &'a T;
    type IntoIter = ::std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.slice.into_iter()
    }
}

trait Producer: ParallelIterator {
    fn produce<C: Consumer<Self::It>>(self, c: &C) -> C::Result;

    fn split_at(&self, mid: usize) -> (Self, Self);
}

impl<'a, T: Send + Sync + 'a> Producer for SliceIter<'a, T> {
    fn produce<C: Consumer<Self::It>>(self, c: &C) -> C::Result {
        let mut folder = c.folder();
        for i in self.slice.into_iter() {
            folder = folder.consume(i)
        }
        folder.result()
    }

    fn split_at(&self, mid: usize) -> (SliceIter<'a, T>, SliceIter<'a, T>) {
        let (left, right) = self.slice.split_at(mid);
        (SliceIter { slice: left }, SliceIter { slice: right })
    }
}

impl<'a, T: Send + Sync> IntoParallelIterator for &'a [T] {
    type Iter = SliceIter<'a, T>;
    fn into_aer_iter(&self) -> SliceIter<'a, T> {
        SliceIter { slice: self }
    }
}

impl<'a, T: Send + Sync> IntoParallelIterator for &'a Vec<T> {
    type Iter = SliceIter<'a, T>;
    fn into_aer_iter(&self) -> SliceIter<'a, T> {
        SliceIter { slice: self }
    }
}

mod tests {
    use super::IntoParallelIterator;
    use super::*;
    use std::iter::Sum;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    #[test]
    fn test_sum() {
        let a = [1; 1000];
        let time1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        //  let b = &a[..];
        let c = (&a[..]).into_aer_iter().sum::<usize>();
        let time2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        println!("{}", c);
        println!("{}", time2 - time1);

        let time1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        //  let b = &a[..];
        let c = (&a[..]).into_aer_iter().sum::<usize>();
        let time2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        println!("{}", c);
        println!("{}", time2 - time1);

        let time1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        //  let b = &a[..];
        let c = (&a[..]).into_aer_iter().sum::<usize>();
        let time2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        println!("{}", c);
        println!("{}", time2 - time1);
        println!("===================");
        let time1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
    }
}
