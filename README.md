# aerogel
Go GMP asynchronous model written in rust


```rust
    let a = [1,2,3];
    let b = [1,2,3];

    let res1 = a.par_iter().sum(); //cpu 并行 现在已经实现
    let res2 = b.con_iter().sum(); //io 并发 tokio 运行时
```
