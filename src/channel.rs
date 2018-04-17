use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A generic declaration for channels.
pub trait Channel<T> {

    /// Just sends a value through the channel.
    /// Must always succeed.
    fn send(&mut self, value: T);

    /// Receives a value. If there is nothing
    /// avaible in the channel, returns `None`.
    fn recv(&mut self) -> Option<T>;

}

/// "Channel Queue" - An implementation of channel using queues.
#[derive(Debug, Clone)]
pub struct Cheue<T> {
    messages: Arc<Mutex<VecDeque<T>>>,
}

impl<T> Cheue<T> {

    /// Creates an empty channel.
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

}

impl<T> Channel<T> for Cheue<T> {

    fn send(&mut self, value: T) {
        self.messages.lock().unwrap().push_back(value);
    }

    fn recv(&mut self) -> Option<T> {
        self.messages.lock().unwrap().pop_front()
    }

}
