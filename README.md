# coro
Rust yield-like coroutines!

See more at docs: https://brunoczim.github.io/coro/

# Example

```rust
extern crate coro;

use coro::{CoFn, CoTasking, Yield, Done};


#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Empty,
    Push(i64),
    Return(Vec<i64>),
}

let consumer = CoFn::new(|scheduler| {
    let producer = scheduler.id_for("producer").unwrap();
    let mut rtrn = Vec::new();
    move |message| match message {
        Msg::Push(x) => {
            rtrn.push(x);
            Yield(producer, Msg::Empty)
        },
        _ => Done(Msg::Return(rtrn.clone()))
    }
});

let producer = CoFn::new(|scheduler| {
    let consumer = scheduler.id_for("consumer").unwrap();
    let mut source = vec![232, -1232, 43];
    move |_| match source.pop() {
        Some(x) => Yield(consumer, Msg::Push(x)),
        _ => Yield(consumer, Msg::Empty),
    }
});

let result = CoTasking::new()
    .task("producer", producer)
    .task("consumer", consumer)
    .run("producer", Msg::Empty);
assert_eq!(result, Msg::Return(vec![43, -1232, 232]));
```
