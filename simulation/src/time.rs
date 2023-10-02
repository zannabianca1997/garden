//! Simulation time

use std::{
    fmt::{Display, Write},
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use either::Either::{self, Left, Right};
use itertools::Itertools;

/// A point in the time of the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Time {
    NegativeInfinity,
    Finite(TimeDelta),
    PositiveInfinity,
}

impl Time {
    const ZERO: Self = Time::Finite(TimeDelta::ZERO);
    const MIN: Self = Time::NegativeInfinity;
    const MAX: Self = Time::PositiveInfinity;

    pub fn difference(&self, from: &Self) -> Option<TimeDelta> {
        match (self, from) {
            (Time::Finite(s), Time::Finite(f)) => Some(*s - *f),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_finite(&self) -> Option<&TimeDelta> {
        if let Self::Finite(v) = self {
            Some(v)
        } else {
            None
        }
    }
    #[must_use]
    pub fn as_finite_mut(&mut self) -> Option<&mut TimeDelta> {
        if let Self::Finite(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the time is [`Finite`].
    ///
    /// [`Finite`]: Time::Finite
    #[must_use]
    pub fn is_finite(&self) -> bool {
        matches!(self, Self::Finite(..))
    }

    /// Returns `true` if the time is [`NegativeInfinity`].
    ///
    /// [`NegativeInfinity`]: Time::NegativeInfinity
    #[must_use]
    pub fn is_negative_infinity(&self) -> bool {
        matches!(self, Self::NegativeInfinity)
    }

    /// Returns `true` if the time is [`PositiveInfinity`].
    ///
    /// [`PositiveInfinity`]: Time::PositiveInfinity
    #[must_use]
    pub fn is_positive_infinity(&self) -> bool {
        matches!(self, Self::PositiveInfinity)
    }
}

impl Add<TimeDelta> for Time {
    type Output = Time;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        match self {
            Time::NegativeInfinity => Time::NegativeInfinity,
            Time::Finite(t) => Time::Finite(t + rhs),
            Time::PositiveInfinity => Time::PositiveInfinity,
        }
    }
}
impl AddAssign<TimeDelta> for Time {
    fn add_assign(&mut self, rhs: TimeDelta) {
        if let Some(t) = self.as_finite_mut() {
            *t += rhs
        }
    }
}
impl Sub<TimeDelta> for Time {
    type Output = Time;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        match self {
            Time::NegativeInfinity => Time::NegativeInfinity,
            Time::Finite(t) => Time::Finite(t - rhs),
            Time::PositiveInfinity => Time::PositiveInfinity,
        }
    }
}
impl SubAssign<TimeDelta> for Time {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        if let Some(t) = self.as_finite_mut() {
            *t -= rhs
        }
    }
}

/// A finite difference between two simulation time points
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(i64);

impl TimeDelta {
    /// The zero time delta
    pub const ZERO: Self = TimeDelta(0);
    /// The smallest time delta
    pub const EPSILON: Self = TimeDelta(1);

    /// The equivalent of a millisecond
    pub const MILLI: Self = TimeDelta(16);
    /// The equivalent of a second
    pub const SECOND: Self = TimeDelta(16 * 1000);
    /// The equivalent of a minute
    pub const MINUTE: Self = TimeDelta(16 * 1000 * 60);
    /// The equivalent of a hour
    pub const HOUR: Self = TimeDelta(16 * 1000 * 60 * 60);
    /// The equivalent of a day
    pub const DAY: Self = TimeDelta(16 * 1000 * 60 * 60 * 24);
    /// The equivalent of a year
    pub const YEAR: Self = TimeDelta(16 * 1000 * 60 * 60 * 24 * 365);

    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
    pub const fn is_positive(&self) -> bool {
        self.0.is_positive()
    }
}

impl Display for TimeDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == TimeDelta::ZERO {
            return 0usize.fmt(f);
        }

        struct Scaling {
            remaining: u64,
            scales: &'static [(u64, &'static str)],
        }

        struct DisplayScale {
            val: Either<u64, f64>,
            unit: &'static str,
        }

        impl Iterator for Scaling {
            type Item = DisplayScale;

            fn next(&mut self) -> Option<Self::Item> {
                let Some(((scale, unit), scales)) = self.scales.split_first() else {
                    return None;
                };
                if self.remaining == 0 {
                    return None;
                }
                self.scales = scales;
                Some(if !scales.is_empty() || self.remaining % scale == 0 {
                    // not the last scale
                    let display = self.remaining / scale;
                    self.remaining = self.remaining % scale;
                    DisplayScale {
                        val: Left(display),
                        unit,
                    }
                } else {
                    // last scale
                    let display = self.remaining as f64 / *scale as f64;
                    self.remaining = 0;
                    DisplayScale {
                        val: Right(display),
                        unit,
                    }
                })
            }
        }

        impl Display for DisplayScale {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.val.fmt(f)?;
                f.write_char(' ')?;
                f.write_str(self.unit)?;
                Ok(())
            }
        }

        const SCALES: [(u64, &str); 6] = [
            (TimeDelta::YEAR.0 as u64, "yr"),
            (TimeDelta::DAY.0 as u64, "d"),
            (TimeDelta::HOUR.0 as u64, "h"),
            (TimeDelta::MINUTE.0 as u64, "m"),
            (TimeDelta::SECOND.0 as u64, "s"),
            (TimeDelta::MILLI.0 as u64, "ms"),
        ];
        let last = if !f.alternate() {
            SCALES
                .into_iter()
                .enumerate()
                .find_map(|(i, (s, _))| (s < self.0.abs_diff(0) / 100).then_some(i))
                .unwrap_or(SCALES.len() - 1)
        } else {
            SCALES.len() - 1
        };

        if self.is_negative() {
            f.write_char('-')?
        }
        Scaling {
            remaining: self.0.abs_diff(0),
            scales: &SCALES[..=last],
        }
        .filter(|d| !matches!(d, DisplayScale { val: Left(0), .. }))
        .format(" ")
        .fmt(f)
    }
}

impl Add for TimeDelta {
    type Output = TimeDelta;

    fn add(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 + rhs.0)
    }
}
impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}
impl Sub for TimeDelta {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 - rhs.0)
    }
}
impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}
impl<T: Into<i64>> Mul<T> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: T) -> Self::Output {
        TimeDelta(self.0 * rhs.into())
    }
}
impl<T: Into<i64>> MulAssign<T> for TimeDelta {
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs.into()
    }
}
