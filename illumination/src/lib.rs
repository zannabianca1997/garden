#![feature(duration_constants)]
#![feature(never_type)]

use std::f64::consts::PI;

use nalgebra::{vector, Vector3};
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
pub struct Illumination {
    /// Duration of the day
    pub day_lenght: Duration,
    /// Highest point of the sun in the sky
    pub solar_noon: Vector3<f64>,
    /// Point where the sun raises
    pub sunrise: Vector3<f64>,
    /// Solar constant, in W/m^2
    pub solar_constant: f64,
}

impl Illumination {
    pub fn new(
        SunSetup {
            day_lenght,
            latitude,
            solar_constant,
        }: SunSetup,
    ) -> Result<Self, !> {
        let latitude = latitude * (PI / 180.);
        Ok(Illumination {
            day_lenght,
            solar_noon: vector![0., -latitude.sin(), latitude.cos()],
            sunrise: vector![1., 0., 0.],
            solar_constant,
        })
    }

    pub fn illuminate(&self, map: Grid<f64>, time: Time) -> Grid<f64> {
        // angle of the sun from the noon
        let sun_theta = (time - Time::ZERO).div_f(self.day_lenght.into());
        // position of the sun in the sky
        let sun_pos = sun_theta.sin() * self.sunrise + sun_theta.cos() * self.solar_noon;
        // energy vector on each face
        let energy_flux = self.solar_constant * sun_pos;

        let mut illumination = Grid::<f64>::new(map.rows(), map.cols());

        // sun from straight above
        if energy_flux.z > 0. {
            illumination.fill(energy_flux.z)
        } else {
            // new initialize at 0.
        }

        if energy_flux.x > 0. {
            // sun on the east side
            for row in 0..map.rows() {
                for col in 0..(map.cols() - 1) {
                    illumination[(row, col)] +=
                        energy_flux.x * (map[(row, col)] - map[(row, col + 1)]).max(0.)
                }
                // last column wrap around
                let col = map.cols() - 1;
                illumination[(row, col)] +=
                    energy_flux.x * (map[(row, col)] - map[(row, 0)]).max(0.)
            }
        } else {
            // sun on the west side
            let flux = -energy_flux.x;
            for row in 0..map.rows() {
                // first column wrap around
                let col = 0;
                illumination[(row, col)] +=
                    flux * (map[(row, col)] - map[(row, map.cols() - 1)]).max(0.);

                for col in 1..map.cols() {
                    illumination[(row, col)] +=
                        flux * (map[(row, col)] - map[(row, col - 1)]).max(0.)
                }
            }
        }

        if energy_flux.y > 0. {
            // first row wraps around
            let row = 0;
            for col in 0..map.cols() {
                illumination[(row, col)] +=
                    energy_flux.y * (map[(row, col)] - map[(map.rows() - 1, col)]).max(0.)
            }
            // sun on the north side
            for row in 1..map.rows() {
                for col in 0..map.cols() {
                    illumination[(row, col)] +=
                        energy_flux.y * (map[(row, col)] - map[(row - 1, col)]).max(0.)
                }
            }
        } else {
            // sun on the south side
            let flux = -energy_flux.y;
            for row in 0..(map.rows() - 1) {
                for col in 0..map.cols() {
                    illumination[(row, col)] +=
                        flux * (map[(row, col)] - map[(row + 1, col)]).max(0.)
                }
            }
            // last row wraps around
            let row = map.rows() - 1;
            for col in 0..map.cols() {
                illumination[(row, col)] += flux * (map[(row, col)] - map[(0, col)]).max(0.)
            }
        }

        illumination
    }
}
