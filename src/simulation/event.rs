use std::error::Error;

use priq::PriorityQueue;

use super::{simulation::Simulation, time::Time};

pub trait Event: Sized {
    type Error: Error;

    /// Run the event
    fn run(self, simulation: &mut Simulation<Self>) -> Result<(), Self::Error>;
}

/// A queue of events
pub struct EventQueue<E: Event>(PriorityQueue<Time, E>);

impl<E: Event> EventQueue<E> {
    /// Add an event to the queue
    pub fn schedule(&mut self, time: Time, event: E) {
        self.0.put(time, event)
    }
    /// Get the next event from the queue
    pub fn next(&mut self) -> Option<(Time, E)> {
        self.0.pop()
    }
    /// Peek at the time of the next event
    pub fn peek_next_time(&self) -> Option<Time> {
        self.0.peek().map(|(time, _)| *time)
    }
}
