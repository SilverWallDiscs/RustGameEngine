//! Cross-thread event delivery primitives.

use crossbeam_channel::{unbounded, Receiver, Sender};

/// Cloneable producer for an engine event stream.
///
/// # Examples
///
/// ~~~rust
/// use engine_core::EventBus;
/// let bus = EventBus::<String>::new();
/// bus.sender().send("asset-ready".to_owned()).unwrap();
/// assert_eq!(bus.drain().next().as_deref(), Some("asset-ready"));
/// ~~~
#[derive(Debug)]
pub struct EventSender<E> { sender: Sender<E> }

impl<E> Clone for EventSender<E> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}

impl<E: Send + 'static> EventSender<E> {
    /// Sends one event to the bus.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EventBus;
    /// let bus = EventBus::<u32>::new();
    /// bus.sender().send(3).unwrap();
    /// assert_eq!(bus.drain().next(), Some(3));
    /// ~~~
    pub fn send(&self, event: E) -> Result<(), crossbeam_channel::SendError<E>> {
        self.sender.send(event)
    }
}

/// Multi-producer, single-consumer event queue.
///
/// # Examples
///
/// ~~~rust
/// use engine_core::EventBus;
/// let bus = EventBus::<u32>::new();
/// bus.publish(42).unwrap();
/// assert_eq!(bus.drain().collect::<Vec<_>>(), vec![42]);
/// ~~~
#[derive(Debug)]
pub struct EventBus<E> { sender: Sender<E>, receiver: Receiver<E> }

impl<E: Send + 'static> EventBus<E> {
    /// Creates an unbounded event channel.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EventBus;
    /// assert!(EventBus::<u32>::new().is_empty());
    /// ~~~
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    /// Publishes an event from this thread.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EventBus;
    /// let bus = EventBus::<u8>::new();
    /// bus.publish(1).unwrap();
    /// ~~~
    pub fn publish(&self, event: E) -> Result<(), crossbeam_channel::SendError<E>> {
        self.sender.send(event)
    }

    /// Creates a producer handle that can be moved to worker threads.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EventBus;
    /// let bus = EventBus::<u8>::new();
    /// let producer = bus.sender();
    /// producer.send(1).unwrap();
    /// ~~~
    pub fn sender(&self) -> EventSender<E> {
        EventSender { sender: self.sender.clone() }
    }

    /// Drains all events queued at the time of the call without blocking.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EventBus;
    /// let bus = EventBus::<u8>::new();
    /// bus.publish(1).unwrap();
    /// assert_eq!(bus.drain().collect::<Vec<_>>(), vec![1]);
    /// ~~~
    pub fn drain(&self) -> impl Iterator<Item = E> + '_ {
        self.receiver.try_iter()
    }

    /// Returns true when no event is pending.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EventBus;
    /// assert!(EventBus::<u8>::new().is_empty());
    /// ~~~
    pub fn is_empty(&self) -> bool { self.receiver.is_empty() }
}

impl<E: Send + 'static> Default for EventBus<E> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::EventBus;
    #[test]
    fn producers_can_send_from_worker_threads() {
        let bus = EventBus::<u32>::new();
        let sender = bus.sender();
        std::thread::spawn(move || sender.send(7).unwrap()).join().unwrap();
        assert_eq!(bus.drain().collect::<Vec<_>>(), vec![7]);
    }
}
