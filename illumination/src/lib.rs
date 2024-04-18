#![feature(duration_constants)]
#![feature(never_type)]

use std::f64::consts::PI;

use field::Field;
use nalgebra::{point, Point3, Unit, UnitVector3, Vector3};
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
            solar_constant: 615.15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Illumination {
    /// Duration of the day
    pub day_lenght: Duration,
    /// Highest point of the sun in the sky
    pub solar_noon: UnitVector3<f64>,
    /// Point where the sun raises
    pub sunrise: UnitVector3<f64>,
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
            solar_noon: UnitVector3::new_unchecked(
                -Vector3::<f64>::y() * latitude.sin() + Vector3::z() * latitude.cos(),
            ),
            sunrise: Vector3::x_axis(),
            solar_constant,
        })
    }

    pub fn illuminate(&self, map: &Field<f64>, time: Time) -> Field<f64> {
        // angle of the sun from the noon
        let sun_theta = (time - Time::ZERO)
            .rem_euclid(self.day_lenght.into())
            .as_time_delta()
            .div_f(self.day_lenght.into())
            * (2. * PI);
        // position of the sun in the sky
        let sun_pos = Unit::new_unchecked(
            sun_theta.sin() * self.sunrise.into_inner()
                + sun_theta.cos() * self.solar_noon.into_inner(),
        );
        // energy vector
        let energy_flux = self.solar_constant * sun_pos.into_inner();

        todo!()
    }
}
