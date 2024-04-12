#![feature(duration_constants)]
#![feature(never_type)]

use serde_with::serde_as;
use sim_time::{Duration, Time};

use grid::Grid;
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct SunSetup {
    /// Duration of the day
    #[serde_as(as = "sim_time::humanized::HumanizeIfNeeded<Duration>")]
    pub day_lenght: Duration,
    /// Latitude, in degrees
    pub latitude: f64,
    /// Solar constant, in W/m^2
    pub solar_constant: f64,
}

impl Default for SunSetup {
    fn default() -> Self {
        Self {
            day_lenght: Duration::DAY,
            latitude: 45.,
            solar_constant: 1367. * 0.45,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Illumination {}

impl Illumination {
    pub fn new(sun: SunSetup) -> Result<Self, !> {
        Ok(Illumination {})
    }

    pub fn illuminate(&self, map: Grid<f64>, time: Time) {}
}
