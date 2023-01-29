use std::{
    fmt::Display,
    num::Wrapping,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

/// The integer type used to save the simulation time
type TimeInt = i64;

/// A time delta of the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(Wrapping<TimeInt>);

impl Time {
    pub const ZERO: Self = Self(Wrapping(0));
    pub const EPSILON: Self = Self(Wrapping(1));
    pub const MIN: Self = Self(Wrapping::<TimeInt>::MIN);
    pub const MAX: Self = Self(Wrapping::<TimeInt>::MAX);

    pub const SECOND: Self = Self(Wrapping(1024));
    pub const MINUTE: Self = Self(Wrapping(Self::SECOND.0 .0.wrapping_mul(60)));
    pub const HOUR: Self = Self(Wrapping(Self::MINUTE.0 .0.wrapping_mul(60)));
    pub const DAY: Self = Self(Wrapping(Self::HOUR.0 .0.wrapping_mul(24)));
    pub const WEEK: Self = Self(Wrapping(Self::DAY.0 .0.wrapping_mul(7)));
    pub const MONTH: Self = Self(Wrapping(Self::DAY.0 .0.wrapping_mul(30)));
    pub const YEAR: Self = Self(Wrapping(Self::DAY.0 .0.wrapping_mul(365)));

    // Splitting various part of the time delta

    /// Split the integer number of years
    fn split_years<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::YEAR;
        let remain = self % Self::YEAR;
        (years.into(), remain)
    }
    /// Split the integer number of months
    fn split_months<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::MONTH;
        let remain = self % Self::MONTH;
        (years.into(), remain)
    }
    /// Split the integer number of weeks
    fn split_weeks<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::WEEK;
        let remain = self % Self::WEEK;
        (years.into(), remain)
    }
    /// Split the integer number of days
    fn split_days<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::DAY;
        let remain = self % Self::DAY;
        (years.into(), remain)
    }
    /// Split the integer number of hours
    fn split_hours<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::HOUR;
        let remain = self % Self::HOUR;
        (years.into(), remain)
    }
    /// Split the integer number of minutes
    fn split_minutes<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::MINUTE;
        let remain = self % Self::MINUTE;
        (years.into(), remain)
    }
    /// Split the integer number of seconds
    fn split_seconds<T: From<TimeInt>>(self) -> (T, Self) {
        let years = self / Self::SECOND;
        let remain = self % Self::SECOND;
        (years.into(), remain)
    }
    /// Split the farctional part of a second
    fn split_fractional_second<T1: From<TimeInt>, T2: From<TimeInt>>(self) -> (T1, T2) {
        (
            self.split_seconds::<TimeInt>().1 .0 .0.into(),
            Self::SECOND.0 .0.into(),
        )
    }
}

// Arithmetic with times

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for Time {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}
impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl SubAssign for Time {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}
impl Neg for Time {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<IntT: Into<TimeInt>> Mul<IntT> for Time {
    type Output = Self;

    fn mul(self, rhs: IntT) -> Self::Output {
        Self(self.0 * Wrapping(rhs.into()))
    }
}
impl<IntT: Into<TimeInt>> MulAssign<IntT> for Time {
    fn mul_assign(&mut self, rhs: IntT) {
        self.0 *= rhs.into()
    }
}

impl Div for Time {
    type Output = TimeInt;

    fn div(self, rhs: Self) -> Self::Output {
        (self.0 / rhs.0).0
    }
}
impl<IntT: Into<TimeInt>> Div<IntT> for Time {
    type Output = Time;

    fn div(self, rhs: IntT) -> Self::Output {
        Time(self.0 / Wrapping(rhs.into()))
    }
}
impl<IntT: Into<TimeInt>> DivAssign<IntT> for Time {
    fn div_assign(&mut self, rhs: IntT) {
        self.0 = (*self / rhs).0;
    }
}
impl Rem for Time {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}
impl RemAssign for Time {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0
    }
}

// Display

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (years, t): (TimeInt, _) = self.split_years();
        let (months, t): (TimeInt, _) = t.split_months();
        let (days, t): (TimeInt, _) = t.split_days();
        let (hours, t): (TimeInt, _) = t.split_hours();
        let (minutes, t): (TimeInt, _) = t.split_minutes();
        let (seconds, sub_second): (TimeInt, _) = t.split_seconds();

        let mut written_something = false;
        for (v, ch) in [
            (years, 'Y'),
            (months, 'M'),
            (days, 'D'),
            (hours, 'h'),
            (minutes, 'm'),
        ] {
            if v != 0 {
                if written_something {
                    write!(f, "{}", f.fill())?;
                }
                write!(f, "{}{}", v, ch)?;
                written_something = true;
            }
        }

        match (seconds, sub_second, written_something) {
            (0, Time::ZERO, false) => write!(f, "{}{}s", f.fill(), 0)?,
            (0, Time::ZERO, true) => (), // do not add unuseful 0s
            (_, Time::ZERO, _) => {
                if written_something {
                    write!(f, "{}", f.fill())?;
                }
                write!(f, "{seconds}s")?;
            }
            (0, _, _) => {
                if written_something {
                    write!(f, "{}", f.fill())?;
                }
                let (n, d): (i64, i64) = sub_second.split_fractional_second();
                if f.alternate() {
                    write!(f, "{n}/{d}s")?;
                } else {
                    let seconds = n as f64 / d as f64;
                    write!(f, "{seconds:.3}s")?;
                }
            }
            (_, _, _) => {
                if written_something {
                    write!(f, "{}", f.fill())?;
                }
                let (n, d): (TimeInt, TimeInt) = sub_second.split_fractional_second();
                if f.alternate() {
                    write!(f, "{seconds} {n}/{d}s")?;
                } else {
                    let seconds = seconds as f64 + n as f64 / d as f64;
                    write!(f, "{seconds:.3}s")?;
                }
            }
        }

        Ok(())
    }
}
