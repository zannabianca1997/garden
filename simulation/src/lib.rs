//! An event driven simulation framework

use std::{cmp::Ordering, collections::BinaryHeap};

use time::{Time, TimeDelta};

/// An event of the simulation
trait Event<State> {
    type Error;
    fn trigger(self, simulation: &mut Simulation<State, Self>)
    where
        Self: Sized;
}

/// The state of the simulation
trait State {}

/// A simulation
#[derive(Debug, Clone)]
struct Simulation<Event, State> {
    time: Time,
    state: State,
    queue: BinaryHeap<QueuedEvent<Event>>,
}

/// An event with an associated time
#[derive(Debug, Clone, Copy)]
struct QueuedEvent<E> {
    /// The finite time at which the event is scheduled
    time: TimeDelta,
    /// The scheduled event
    event: E,
}

impl<E> PartialEq for QueuedEvent<E> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl<E> Eq for QueuedEvent<E> {}
impl<E> PartialOrd for QueuedEvent<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // reverse the order so it become a min-heap
        // putting the smallest time event before the others
        self.time.partial_cmp(&other.time).map(Ordering::reverse)
    }
}
impl<E> Ord for QueuedEvent<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // reverse the order so it become a min-heap
        // putting the smallest time event before the others
        self.time.cmp(&other.time).reverse()
    }
}

pub mod time;
