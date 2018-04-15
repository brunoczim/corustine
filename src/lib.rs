//! Repository: https://github.com/brunoczim/corustine
//! This crate implements coroutines in rust.
//! Please note that, in this implementation, one
//! cannot yield a function inside some control flow structre.
//! Instead, the function must return either a `Yield` or a
//! `Done`. Note also that a coroutine may not be an actual function.
//! The coroutine only has to implement the trait `Coroutine<T>`.
//! Actually, functions by themselves are not coroutines.
//! Instead, they must be wrapped with `CoFn` or a custom wrapper.
//! Look at this example.
//! ```rust
//! extern crate corustine;
//!
//! use corustine::{CoFn, CoTasking, Yield, Done};
//!
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum Msg {
//!     Empty,
//!     Push(i64),
//!     Return(Vec<i64>),
//! }
//!
//! let consumer = CoFn::new(|scheduler| {
//!     let producer = scheduler.id_for("producer").unwrap();
//!     let mut rtrn = Vec::new();
//!     move |message| match message {
//!         Msg::Push(x) => {
//!             rtrn.push(x);
//!             Yield(producer, Msg::Empty)
//!         },
//!         _ => Done(Msg::Return(rtrn.clone()))
//!     }
//! });
//!
//! let producer = CoFn::new(|scheduler| {
//!     let consumer = scheduler.id_for("consumer").unwrap();
//!     let mut source = vec![232, -1232, 43];
//!     move |_| match source.pop() {
//!         Some(x) => Yield(consumer, Msg::Push(x)),
//!         _ => Yield(consumer, Msg::Empty),
//!     }
//! });
//!
//! let result = CoTasking::new()
//!     .task("producer", producer)
//!     .task("consumer", consumer)
//!     .run("producer", Msg::Empty);
//! assert_eq!(result, Msg::Return(vec![43, -1232, 232]));
//! ```

use std::cmp::{Ordering, Ord};
use std::marker::PhantomData;
use std::{fmt, mem};

pub use self::Pause::*;

/// Identifies a single coroutine instance in a scheduler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoId(usize);

/// Must be returned from a single call of `resume`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pause<T> {
    /// This means that the coroutine indeed paused.
    /// It contains the next function and a message.
    Yield(CoId, T),
    /// This means that the coroutine actually stopped.
    /// This will make all tasks stop. It contains a final
    /// value.
    Done(T),
}

/// This trait defines a coroutine in order to be used with
/// this crate.
pub trait Coroutine<T>: fmt::Debug {

    /// Initializes self, so it can, as an example,
    /// grab all necessary ids.
    fn init(&mut self, scheduler: &Scheduler<T>);

    /// Resumes/starts this coroutine.
    fn resume(&mut self, message: T) -> Pause<T>;

}

/// A boilerplate reduction for corroutines implemented as
/// regular rust functions.
pub struct CoFn<I, R, T>
where
    I: FnOnce(&Scheduler<T>) -> R,
    R: FnMut(T) -> Pause<T>,
{
    inner: CoFnInner<I, R>,
    _marker: PhantomData<T>,
}

impl<I, R, T> CoFn<I, R, T>
where
    I: FnOnce(&Scheduler<T>) -> R,
    R: FnMut(T) -> Pause<T>,
{

    /// Creates a new coroutine function from a given
    /// initializer.
    pub fn new(init: I) -> Self {
        Self {
            inner: CoFnInner::Init(init),
            _marker: PhantomData,
        }
    }

}

impl<I, R, T> Coroutine<T> for CoFn<I, R, T>
where
    I: FnOnce(&Scheduler<T>) -> R,
    R: FnMut(T) -> Pause<T>,
{

    fn init(&mut self, scheduler: &Scheduler<T>) {
        let init = mem::replace(&mut self.inner, CoFnInner::None);
        match init {
            CoFnInner::Init(f) => {
                self.inner = CoFnInner::Resume(f(scheduler))
            },
            _ => panic!("Internal coroutine function state error"),
        }
    }

    fn resume(&mut self, message: T) -> Pause<T> {
        match &mut self.inner {
            &mut CoFnInner::Resume(ref mut f) => {
                f(message)
            },
            _ => panic!("Internal coroutine function state error"),
        }
    }

}

impl<I, R, T> fmt::Debug for CoFn<I, R, T>
where
    I: FnOnce(&Scheduler<T>) -> R,
    R: FnMut(T) -> Pause<T>,
{

    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "CoFn@{:x}", self as *const Self as usize)
    }

}

enum CoFnInner<I, R> {
    None,
    Init(I),
    Resume(R),
}


#[derive(Debug)]
struct Task<T> {
    name: &'static str,
    coro: Option<Box<Coroutine<T>>>,
}

/// Builds a Co-operative tasking with some given coroutines.
#[derive(Debug)]
pub struct CoTasking<T> {
    tasks: Vec<Task<T>>,
}

impl<T> CoTasking<T> {

    /// A fresh zeroed builder.
    pub fn new() -> Self {
        Self {tasks: Vec::with_capacity(16)}
    }

    /// Adds a task with the given name and coroutine.
    pub fn task<C>(mut self, name: &'static str, coro: C) -> Self
    where
        C: Coroutine<T> + 'static
    {
        let idx = if self.tasks.len() > 0 {
            let mut top = self.tasks.len() - 1;
            let mut bottom = 0;
            loop {
                let i = top + bottom >> 1;
                match self.tasks[i].name.cmp(name) {
                    Ordering::Equal => {
                        // a corner case; if we already have it,
                        // just update the coroutine
                        self.tasks[i].coro = Some(Box::new(coro));
                        return self;
                    },
                    Ordering::Greater => bottom = i + 1,
                    Ordering::Less => if i == 0 {
                        break i
                    } else {
                        top = i - 1
                    },
                }
                if bottom > top {
                    break bottom
                }
            }
        } else {
            0
        };
        self.tasks.insert(idx, Task {
            name,
            coro: Some(Box::new(coro)),
        });
        self
    }

    /// Runs starting from the given coroutine name and an
    /// initial message.
    pub fn run(self, starter: &'static str, init: T) -> T {
        let mut scheduler = Scheduler {
            tasks: self.tasks.into_boxed_slice(),
            next_id: CoId(0), // arbitrary temporary value
            next_message: Some(init),
        };
        let id = scheduler.id_for(starter)
            .expect("The starter with the given name was not found");
        scheduler.next_id = id;
        scheduler.run()
    }

}

/// A type that manages a co-operative tasking execution.
/// Mostly invisible.
#[derive(Debug)]
pub struct Scheduler<T> {
    tasks: Box<[Task<T>]>,
    next_id: CoId,
    next_message: Option<T>,
}

impl<T> Scheduler<T> {

    /// Returns the identification for the given coroutine name.
    pub fn id_for(&self, name: &'static str) -> Option<CoId> {
        let mut top = self.tasks.len() - 1;
        let mut bottom = 0;
        loop {
            let i = top + bottom >> 1;
            match self.tasks[i].name.cmp(name) {
                Ordering::Equal => break Some(CoId(i)),
                Ordering::Greater => bottom = i + 1,
                Ordering::Less => if i == 0 {
                    break None
                } else {
                    top = i - 1
                },
            }
            if bottom > top {
                break None
            }
        }
    }

    fn run(mut self) -> T {
        for i in 0..self.tasks.len() {
            let mut coro = self.tasks[i].coro.take().unwrap();
            coro.init(&self);
            self.tasks[i].coro = Some(coro);
        }
        loop {
            let msg = self.next_message.take().unwrap();
            let id = self.next_id;
            let mut coro = self.tasks[id.0].coro.take().unwrap();
            let pause = coro.resume(msg);
            self.tasks[id.0].coro = Some(coro);
            match pause {
                Yield(next_id, next_msg) => {
                    self.next_id = next_id;
                    self.next_message = Some(next_msg);
                },
                Done(res) => break res,
            }
        }
    }

}
