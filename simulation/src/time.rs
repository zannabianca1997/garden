//! Simulation time

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use thiserror::Error;

/// A point in the time of the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(u64);

impl Time {
    const ZERO: Self = Time(0);
    const INFINITY: Self = Time(u64::MAX);
    const MIN: Self = Time::ZERO;
    const MAX: Self = Time::INFINITY;

    pub fn difference(&self, from: &Self) -> Option<TimeDelta> {
        if self.is_finite() && from.is_finite() {
            Some(TimeDelta(if self > from {
                (self.0 - from.0).try_into().ok()?
            } else {
                i64::try_from(from.0 - self.0).ok()?.checked_neg()?
            }))
        } else {
            None
        }
    }

    /// Returns `true` if the time is [`Finite`].
    ///
    /// [`Finite`]: Time::Finite
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.0 < u64::MAX
    }
}

impl Add<TimeDelta> for Time {
    type Output = Time;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        if self.is_finite() {
            Time(if cfg!(debug_assertions) {
                self.0
                    .checked_add_signed(rhs.0)
                    .expect("Overflow while adding time delta")
            } else {
                self.0.wrapping_add_signed(rhs.0)
            })
        } else {
            self
        }
    }
}
impl AddAssign<TimeDelta> for Time {
    fn add_assign(&mut self, rhs: TimeDelta) {
        *self = *self + rhs
    }
}
impl Sub<TimeDelta> for Time {
    type Output = Time;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        self + rhs.neg()
    }
}
impl SubAssign<TimeDelta> for Time {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        *self = *self - rhs
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
impl Neg for TimeDelta {
    type Output = TimeDelta;

    fn neg(self) -> Self::Output {
        TimeDelta(self.0.neg())
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

/// A positive, non zero time delta
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveTimeDelta(TimeDelta);

impl PositiveTimeDelta {
    pub fn get(self) -> TimeDelta {
        self.0
    }
}

impl std::ops::Deref for PositiveTimeDelta {
    type Target = TimeDelta;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PositiveTimeDelta> for TimeDelta {
    fn from(value: PositiveTimeDelta) -> Self {
        value.get()
    }
}

impl TryFrom<TimeDelta> for PositiveTimeDelta {
    type Error = NotPositiveTimeDelta;

    fn try_from(value: TimeDelta) -> Result<Self, Self::Error> {
        if value < TimeDelta::ZERO {
            Err(NotPositiveTimeDelta::Negative)
        } else if value == TimeDelta::ZERO {
            Err(NotPositiveTimeDelta::Zero)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NotPositiveTimeDelta {
    #[error("The time delta is negative")]
    Negative,
    #[error("The time delta is zero")]
    Zero,
}

mod display {
    use std::fmt::{Display, Write};

    use either::Either::{self, Left, Right};
    use itertools::Itertools;

    use super::{Time, TimeDelta};

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

    impl Display for TimeDelta {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if *self == TimeDelta::ZERO {
                return 0usize.fmt(f);
            }

            if self.is_negative() {
                f.write_char('-')?
            }

            print_u64(f, self.0.abs_diff(0))
        }
    }

    impl Display for Time {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if *self == Time::ZERO {
                return 0usize.fmt(f);
            }
            if self.is_finite() {
                print_u64(f, self.0)
            } else {
                f.write_str("inf")
            }
        }
    }

    fn print_u64(f: &mut std::fmt::Formatter<'_>, time: u64) -> Result<(), std::fmt::Error> {
        let last = if !f.alternate() {
            SCALES
                .into_iter()
                .enumerate()
                .find_map(|(i, (s, _))| (s < time / 100).then_some(i))
                .unwrap_or(SCALES.len() - 1)
        } else {
            SCALES.len() - 1
        };
        Scaling {
            remaining: time,
            scales: &SCALES[..=last],
        }
        .filter(|d| !matches!(d, DisplayScale { val: Left(0), .. }))
        .format(" ")
        .fmt(f)
    }
}
