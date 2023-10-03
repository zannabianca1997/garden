//! An event driven simulation framework

use std::{cmp::Ordering, collections::BinaryHeap};

use thiserror::Error;
use time::{PositiveTimeDelta, Time};

/// An event of the simulation
pub trait Event<State> {
    type Error;
    fn trigger(self, simulation: &mut Simulation<Self, State>) -> Result<(), Self::Error>
    where
        Self: Sized;
}

/// A simulation
#[derive(Debug, Clone)]
pub struct Simulation<Event, State> {
    time: Time,
    state: State,
    events: BinaryHeap<QueuedEvent<Event>>,
}

impl<E, S> Simulation<E, S> {
    /// Create a new simulation at negative infinity
    pub fn new(state: S) -> Self {
        Self {
            time: Time::ZERO,
            state,
            events: BinaryHeap::new(),
        }
    }

    pub fn time(&self) -> Time {
        self.time
    }

    pub fn ended(&self) -> bool {
        !self.time.is_finite()
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn events_count(&self) -> usize {
        self.events.len()
    }

    pub fn events_clear(&mut self) {
        self.events.clear()
    }

    pub fn events_is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn schedule(
        &mut self,
        event: E,
        delay: PositiveTimeDelta,
    ) -> Result<(), CannotScheduleAtInfinite> {
        if !self.time.is_finite() {
            return Err(CannotScheduleAtInfinite);
        }
        let time = self.time + delay.get();
        self.events.push(QueuedEvent { time, event });
        Ok(())
    }
}

impl<E, S> Simulation<E, S>
where
    E: Event<S>,
{
    /// Advance monotically the simulation
    pub fn step(&mut self) -> Result<Time, E::Error> {
        let Some(QueuedEvent { time, event }) = self.events.pop() else {
            // no more events, going to infinity
            self.time = Time::INFINITY;
            return Ok(self.time);
        };
        self.time = time;
        event.trigger(self)?;
        Ok(self.time)
    }

    /// Advance the simulation until a given time
    pub fn run_until(&mut self, time: Time) -> Result<(), E::Error> {
        while self.time < time {
            self.step()?;
        }
        Ok(())
    }
    /// Advance the simulation for a given time
    pub fn run_for(&mut self, time: PositiveTimeDelta) -> Result<(), E::Error> {
        self.run_until(self.time + time.get())
    }
    /// Advance the simulation until no more events happens
    pub fn run(&mut self) -> Result<(), E::Error> {
        self.run_until(Time::INFINITY)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Cannot schedule an event at infinite time (simulation has ended)")]
pub struct CannotScheduleAtInfinite;

/// An event with an associated time
#[derive(Debug, Clone, Copy)]
struct QueuedEvent<E> {
    /// The finite time at which the event is scheduled
    time: Time,
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
