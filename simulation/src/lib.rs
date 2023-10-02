//! An event driven simulation framework
#![feature(iter_map_windows)]

/// An event of the simulation
trait Event {}

/// The state of the simulation
trait State {}

pub mod time;
