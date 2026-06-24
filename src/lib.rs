#![feature(lazy_cell)]
#![feature(thread_id_value)]
#![feature(thread_local)]

pub mod iter;
pub mod runtime;
mod tests {
    use super::*;

    #[test]
    fn test_processor() {
        let mut a = vec![1, 2, 3, 4, 5];
        drain(&mut a);
        println!("{:?}", a);
    }

    fn drain(a: &mut Vec<i32>) {
        for _ in a.drain(..a.len() - 1) {}
    }
}
