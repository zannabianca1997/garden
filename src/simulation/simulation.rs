use thiserror::Error;

use super::{
    event::{Event, EventQueue},
    state::State,
    time::Time,
};

/// An entire simulation, with entities and event queue
pub struct Simulation<E: Event<State = S>, S: State> {
    /// The current time of the simulation
    time: Time,
    /// The queue of events due to happen
    events: EventQueue<E>,
    /// The state of the simulation
    state: S,
}

impl<E: Event<State = S>, S: State> Simulation<E, S> {
    /// Time elapsed from the start of the simulation
    pub const fn time(&self) -> Time {
        self.time
    }
    /// Step the simulation of an unspecified but monotonous time. Return the time elapsed.
    pub fn step(&mut self) -> Result<Time, TimedSimulationError<E>> {
        // recovering the next event if exist
        let (next_time, event) = self
            .events
            .next()
            .ok_or(SimulationError::NoMoreEvents.at(self.time()))?;
        // change the time and run the event
        self.time = next_time;
        event
            .run(self) // Run the event
            .map(|_| next_time) // Return the time if ok
            .map_err(|err| SimulationError::EventError(err).at(next_time)) // Add the time to the error
    }
    /// Run the simulation until a given timestamp
    pub fn run_until(&mut self, time: Time) -> Result<Time, TimedSimulationError<E>> {
        // run until the next event would be after the given timestamp
        while self
            .events
            .peek_next_time()
            .ok_or(SimulationError::NoMoreEvents.at(self.time()))?
            < time
        {
            self.step()?;
        }
        // advance to the required time
        self.time = time;
        Ok(self.time())
    }
    /// Run the simulation for a given time
    pub fn run_for(&mut self, time: Time) -> Result<Time, TimedSimulationError<E>> {
        self.run_until(self.time() + time)
    }
    /// Run the simulation while a condition is met
    pub fn run_while(
        &mut self,
        condition: fn(&Self) -> bool,
    ) -> Result<Time, TimedSimulationError<E>> {
        while condition(self) {
            self.step()?;
        }
        Ok(self.time())
    }
    /// Get the state of the simulation
    pub const fn state(&self) -> &S {
        &self.state
    }
    /// Get the mutable state of the simulation
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }
}

/// An error occurred during a simulation, with the time at which it occurred
#[derive(Debug, Error)]
#[error("Error during the simulation at time {time}: {error}")]
struct TimedSimulationError<E: Event> {
    time: Time,
    #[source]
    error: SimulationError<E>,
}

/// A generic error occurred during a simulation
#[derive(Debug, Error)]
enum SimulationError<E: Event> {
    #[error("Event queue emptied")]
    NoMoreEvents,

    #[error(transparent)]
    EventError(E::Error),
}

impl<E: Event> SimulationError<E> {
    /// Add a time to this error
    fn at(self, time: Time) -> TimedSimulationError<E> {
        TimedSimulationError { time, error: self }
    }
}
