use std::cmp::{Ordering, Ord};
use std::fmt;

pub use self::Pause::*;

/// Must be returned from a single call of `resume`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pause<I, T> {
    /// This means that the coroutine indeed paused.
    /// It contains the next function identification.
    Yield(I),
    /// This means that the coroutine actually stopped.
    /// This will make all tasks stop. It contains a final
    /// value.
    Done(T),
}

struct Coroutine<I, T> {
    inner: Box<FnMut() -> Pause<I, T>>,
}

impl<I, T> Coroutine<I, T> {

    fn new<F: FnMut() -> Pause<I, T> + 'static>(fun: F) -> Self {
        Self {
            inner: Box::new(fun),
        }
    }

}

impl<I, T> fmt::Debug for Coroutine<I, T> {

    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "coroutine@{:?}", &self.inner as *const _)
    }

}

#[derive(Debug)]
struct Task<I, T> {
    name: I,
    coro: Coroutine<I, T>,
}

/// Builds a Co-operative tasking with some given coroutines.
#[derive(Debug)]
pub struct CoTasking<I, T> {
    tasks: Vec<Task<I, T>>,
}

impl<I, T> CoTasking<I, T>
where
    I: Ord
{

    /// A fresh zeroed builder.
    pub fn new() -> Self {
        Self {tasks: Vec::with_capacity(16)}
    }

    /// Adds a task with the given name and coroutine.
    pub fn task<F: 'static>(mut self, name: I, coro: F) -> Self
    where
        F: FnMut() -> Pause<I, T>
    {
        let idx = if self.tasks.len() > 0 {
            let mut top = self.tasks.len() - 1;
            let mut bottom = 0;
            loop {
                let i = top + bottom >> 1;
                match self.tasks[i].name.cmp(&name) {
                    Ordering::Equal => {
                        // a corner case; if we already have it,
                        // just update the coroutine
                        self.tasks[i].coro = Coroutine::new(coro);
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
            coro: Coroutine::new(coro),
        });
        self
    }

    /// Runs starting from the given coroutine name and an
    /// initial message.
    pub fn run(self, starter: I) -> T {
        let scheduler = Scheduler {
            tasks: self.tasks.into_boxed_slice(),
            next: starter,
        };
        scheduler.run()
    }

}

/// A type that manages a co-operative tasking execution.
/// Mostly invisible.
#[derive(Debug)]
struct Scheduler<I, T> {
    tasks: Box<[Task<I, T>]>,
    next: I,
}

impl<I, T> Scheduler<I, T>
where
    I: Ord
{

    fn search(&mut self) -> Option<&mut Coroutine<I, T>> {
        let mut top = self.tasks.len() - 1;
        let mut bottom = 0;
        loop {
            let i = top + bottom >> 1;
            match self.tasks[i].name.cmp(&self.next) {
                Ordering::Equal => break Some(&mut self.tasks[i].coro),
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
        loop {
            self.next = {
                let fun = &mut self
                    .search()
                    .expect("The given coroutine id was not found")
                    .inner;
                match fun() {
                    Yield(next) => next,
                    Done(res) => break res,
                }
            };
        }
    }

}
