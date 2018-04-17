//! Repository: <https://github.com/brunoczim/corustine>
//!
//! This crate implements coroutines in rust.
//! Please note that, in this implementation, one
//! cannot yield a function inside some control flow structre.
//! Instead, the function must return either a `Yield` or a
//! `Done`. Also, yielding only returns a continuation, but
//! no value. This means that all communication is made through
//! channels.
//! Look at this example.
//! ```rust
//! extern crate corustine;
//!
//! use corustine::{
//!     task::{CoTasking, Yield, Done},
//!     channel::{Channel, Cheue},
//! };
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! enum Task {
//!     Producer,
//!     Consumer,
//! }
//!
//! let mut ch1 = Cheue::new();
//!
//! let producer = {
//!     let mut ch1 = ch1.clone();
//!     let mut m = 1;
//!     let mut n = 0;
//!     move || {
//!         ch1.send(m);
//!         let tmp = n;
//!         n = m;
//!         m += tmp;
//!         Yield(Task::Consumer)
//!     }
//! };
//!
//! let consumer = {
//!     let mut seq = Vec::new();
//!     let lim = 10;
//!     move || if seq.len() >= lim {
//!         Done(seq.clone())
//!     } else {
//!         seq.push(ch1.recv().unwrap());
//!         Yield(Task::Producer)
//!     }
//! };
//!
//! let result = CoTasking::new()
//!     .task(Task::Consumer, consumer)
//!     .task(Task::Producer, producer)
//!     .run(Task::Producer);
//!
//! assert_eq!(result, &[1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
//! ```

/// This module supplies a channel trait and some implementations.
pub mod channel;

/// This module supplies the facilities for creating tasks.
pub mod task;
