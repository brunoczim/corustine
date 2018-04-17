# coro
Rust yield-like coroutines!

See more at docs: https://brunoczim.github.io/corustine/corustine/

# Example

```rust
extern crate corustine;

use corustine::{
    task::{CoTasking, Yield, Done},
    channel::{Channel, Cheue},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Task {
    Producer,
    Consumer,
}

let mut ch1 = Cheue::new();

let producer = {
    let mut ch1 = ch1.clone();
    let mut m = 1;
    let mut n = 0;
    move || {
        ch1.send(m);
        let tmp = n;
        n = m;
        m += tmp;
        Yield(Task::Consumer)
    }
};

let consumer = {
    let mut seq = Vec::new();
    let lim = 10;
    move || if seq.len() >= lim {
        Done(seq.clone())
    } else {
        seq.push(ch1.recv().unwrap());
        Yield(Task::Producer)
    }
};

let result = CoTasking::new()
    .task(Task::Consumer, consumer)
    .task(Task::Producer, producer)
    .run(Task::Producer);

assert_eq!(result, &[1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
```
