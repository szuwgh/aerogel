use std::sync::LazyLock;

static RT: LazyLock<Executor> = LazyLock::new(|| Executor::new());

struct Executor {
    local_queue: TaskQueue,
}

impl Executor {
    fn new() -> Self {
        todo!()
    }
}

struct TaskQueue {}

struct Waker {}
