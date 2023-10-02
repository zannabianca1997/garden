//! An event driven simulation framework

/// An event of the simulation
trait Event {}

/// The state of the simulation
trait State {}

mod time {
    //! Simulation time

    use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

    /// A point in the time of the simulation
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Time {
        NegativeInfinity,
        Finite(TimeDelta),
        PositiveInfinity,
    }

    impl Time {
        const ZERO: Self = Time::Finite(TimeDelta::ZERO);
        const MIN: Self = Time::NegativeInfinity;
        const MAX: Self = Time::PositiveInfinity;

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
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TimeDelta(i64);

    impl TimeDelta {
        /// The zero time delta
        const ZERO: Self = TimeDelta(0);
        /// The smallest time delta
        const EPSILON: Self = TimeDelta(1);
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
}