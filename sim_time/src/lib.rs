//! Implement time for the simulation, with 1/1024 of a second accurancy

use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use arbitrary::{size_hint, Arbitrary, Unstructured};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A positive difference between two points in time
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(TimeDelta);

impl Duration {
    pub const ZERO: Duration = Duration(TimeDelta::ZERO);
    pub const EPSILON: Duration = Duration(TimeDelta::EPSILON);
    pub const SECOND: Duration = Duration(TimeDelta::SECOND);
    pub const MINUTE: Duration = Duration(TimeDelta::MINUTE);
    pub const HOUR: Duration = Duration(TimeDelta::HOUR);
    pub const DAY: Duration = Duration(TimeDelta::DAY);
    pub const YEAR: Duration = Duration(TimeDelta::YEAR);

    pub const MIN: Duration = Duration(TimeDelta::ZERO);
    pub const MAX: Duration = Duration(TimeDelta::MAX);
}

#[derive(Debug, Clone, Copy, Error)]
/// Negative time delta converted to duration
#[error("Negative time delta {0:?} converted to duration")]
pub struct NegativeTimeDelta(TimeDelta);

impl TryFrom<TimeDelta> for Duration {
    type Error = NegativeTimeDelta;

    fn try_from(value: TimeDelta) -> Result<Self, Self::Error> {
        if value >= TimeDelta::ZERO {
            Ok(Self(value))
        } else {
            Err(NegativeTimeDelta(value))
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Self(TimeDelta::from(self).add(rhs.0))
    }
}
impl Add<TimeDelta> for Duration {
    type Output = TimeDelta;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        rhs.add(TimeDelta::from(self))
    }
}
impl Add<Time> for Duration {
    type Output = Time;

    fn add(self, rhs: Time) -> Self::Output {
        rhs.offset(self.into())
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for Duration {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}
impl Sub<TimeDelta> for Duration {
    type Output = TimeDelta;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        self.0 - rhs
    }
}

impl Mul<u64> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u64) -> Self::Output {
        Self(self.0 * rhs as i64)
    }
}
impl Mul<f64> for Duration {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        assert!(
            rhs >= 0.,
            "Duration must be multiplied by a positive number"
        );
        Self(self.0 * rhs)
    }
}

impl MulAssign<u64> for Duration {
    fn mul_assign(&mut self, rhs: u64) {
        *self = *self * rhs
    }
}
impl MulAssign<f64> for Duration {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs
    }
}

impl Div<u64> for Duration {
    type Output = Duration;

    fn div(self, rhs: u64) -> Self::Output {
        Self(self.0 / rhs as i64)
    }
}
impl DivAssign<u64> for Duration {
    fn div_assign(&mut self, rhs: u64) {
        *self = *self / rhs
    }
}

impl Rem for Duration {
    type Output = Duration;

    fn rem(self, rhs: Self) -> Self::Output {
        (self.0 % rhs.0)
            .try_into()
            .expect("The result should be positive")
    }
}

impl<'a> Arbitrary<'a> for Duration {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary().map(TimeDelta::abs)
    }
}

/// A difference between two point in time
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(i64);

impl TimeDelta {
    pub const ZERO: TimeDelta = TimeDelta(0);
    pub const EPSILON: TimeDelta = TimeDelta(1);
    pub const SECOND: TimeDelta = TimeDelta(1024);
    pub const MINUTE: TimeDelta = TimeDelta(1024 * 60);
    pub const HOUR: TimeDelta = TimeDelta(1024 * 60 * 60);
    pub const DAY: TimeDelta = TimeDelta(1024 * 60 * 60 * 24);
    pub const YEAR: TimeDelta = TimeDelta(1024 * 60 * 60 * 24 * 365);

    pub const MIN: TimeDelta = TimeDelta(i64::MIN);
    pub const MAX: TimeDelta = TimeDelta(i64::MAX);

    pub fn div_f(self, rhs: Self) -> f64 {
        self.0 as f64 / rhs.0 as f64
    }
    pub fn div_i(&self, rhs: Self) -> i64 {
        self.0 / rhs.0
    }

    pub fn abs(self) -> Duration {
        Duration(Self(self.0.abs()))
    }
}

impl From<Duration> for TimeDelta {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl Add for TimeDelta {
    type Output = TimeDelta;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Add<Duration> for TimeDelta {
    type Output = TimeDelta;

    fn add(self, rhs: Duration) -> Self::Output {
        self.add(TimeDelta::from(rhs))
    }
}
impl Add<Time> for TimeDelta {
    type Output = Time;

    fn add(self, rhs: Time) -> Self::Output {
        rhs.offset(self)
    }
}

impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
impl AddAssign<Duration> for TimeDelta {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs
    }
}

impl Neg for TimeDelta {
    type Output = TimeDelta;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Sub for TimeDelta {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl Sub<Duration> for TimeDelta {
    type Output = TimeDelta;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.sub(TimeDelta::from(rhs))
    }
}

impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        *self = *self - rhs
    }
}
impl SubAssign<Duration> for TimeDelta {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs
    }
}

impl Mul<i64> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl Mul<TimeDelta> for i64 {
    type Output = TimeDelta;

    fn mul(self, rhs: TimeDelta) -> Self::Output {
        rhs * self
    }
}

impl Mul<f64> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: f64) -> Self::Output {
        Self((self.0 as f64 * rhs) as _)
    }
}
impl Mul<TimeDelta> for f64 {
    type Output = TimeDelta;

    fn mul(self, rhs: TimeDelta) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<i64> for TimeDelta {
    fn mul_assign(&mut self, rhs: i64) {
        *self = *self * rhs
    }
}
impl MulAssign<f64> for TimeDelta {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs
    }
}

impl Div<i64> for TimeDelta {
    type Output = TimeDelta;

    fn div(self, rhs: i64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl DivAssign<i64> for TimeDelta {
    fn div_assign(&mut self, rhs: i64) {
        *self = *self / rhs
    }
}

impl Rem for TimeDelta {
    type Output = TimeDelta;

    fn rem(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 % rhs.0)
    }
}
impl RemAssign for TimeDelta {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs
    }
}

impl<'a> Arbitrary<'a> for TimeDelta {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let arb = u32::arbitrary(u)?;

        let factor_bits = arb & ((1 << 21) - 1);
        let rem_bits = arb >> 21;

        // choosing a unit so values around the 0 are well tested
        let unit = [
            TimeDelta::SECOND,
            TimeDelta::MINUTE,
            TimeDelta::DAY,
            TimeDelta::HOUR,
            TimeDelta::YEAR,
        ][(rem_bits % 5) as usize];
        // checking for all units a span of 1024 from 0, with a step of 1/1024
        let factor = (factor_bits as f64 / 1024.) - 1024.;

        Ok(factor * unit)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        u32::size_hint(depth)
    }
}

/// A position in the simulation time
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(TimeDelta);

impl Time {
    pub const ZERO: Time = Time(TimeDelta::ZERO);
    pub const EPSILON: Time = Time(TimeDelta::EPSILON);
    pub const SECOND: Time = Time(TimeDelta::SECOND);
    pub const MINUTE: Time = Time(TimeDelta::MINUTE);
    pub const HOUR: Time = Time(TimeDelta::HOUR);
    pub const DAY: Time = Time(TimeDelta::DAY);
    pub const YEAR: Time = Time(TimeDelta::YEAR);

    pub const MIN: Time = Time(TimeDelta::MIN);
    pub const MAX: Time = Time(TimeDelta::MAX);

    /// Add a time delta to this moment in time
    pub fn offset(self, delta: TimeDelta) -> Time {
        Time(self.0.add(delta))
    }
}

impl Add<TimeDelta> for Time {
    type Output = Time;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        self.offset(rhs)
    }
}
impl Add<Duration> for Time {
    type Output = Time;

    fn add(self, rhs: Duration) -> Self::Output {
        self.offset(rhs.into())
    }
}

impl AddAssign<TimeDelta> for Time {
    fn add_assign(&mut self, rhs: TimeDelta) {
        *self = *self + rhs
    }
}
impl AddAssign<Duration> for Time {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs
    }
}

impl Sub for Time {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}
impl Sub<TimeDelta> for Time {
    type Output = Time;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl Sub<Duration> for Time {
    type Output = Time;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<TimeDelta> for Time {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        *self = *self - rhs
    }
}
impl SubAssign<Duration> for Time {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs
    }
}

impl<'a> Arbitrary<'a> for Time {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary().map(Self)
    }
}

pub mod humanized {
    use std::fmt::{Debug, Display, Write};
    use std::num::ParseFloatError;
    use std::str::FromStr;

    use lazy_regex::{regex_captures, regex_is_match};
    use serde::Deserialize;
    use serde::{de::Error as _, Deserializer, Serializer};
    use serde_with::{DeserializeAs, IfIsHumanReadable, SerializeAs};
    use thiserror::Error;

    use crate::NegativeTimeDelta;

    use super::{Duration, Time, TimeDelta};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Wrap a type and provide helper functions to parse human representations
    pub struct Humanized<T>(T);

    macro_rules! humanized_from_into {
        ($t:ty) => {
            impl From<$t> for Humanized<$t> {
                fn from(value: $t) -> Self {
                    Humanized(value)
                }
            }
            impl From<Humanized<$t>> for $t {
                fn from(value: Humanized<$t>) -> Self {
                    value.0
                }
            }
        };
    }
    humanized_from_into! {Time}
    humanized_from_into! {TimeDelta}
    humanized_from_into! {Duration}

    impl Display for Humanized<TimeDelta> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut t = self.0;
            let mut spacing = false;

            if t == TimeDelta::ZERO {
                return f.write_str("0");
            }

            for (unit, unit_name) in [
                (TimeDelta::YEAR, "y"),
                (TimeDelta::DAY, "d"),
                (TimeDelta::HOUR, "h"),
                (TimeDelta::MINUTE, "m"),
            ] {
                let n = t.div_i(unit);
                t %= unit;
                if n != 0 {
                    if spacing {
                        f.write_char(' ')?;
                    }
                    write!(f, "{n}{unit_name}")?;
                    spacing = true;
                }
            }

            if t != TimeDelta::ZERO {
                let secs = t.div_f(TimeDelta::SECOND);
                if spacing {
                    f.write_char(' ')?;
                }
                write!(f, "{secs}s")?;
            }

            Ok(())
        }
    }
    impl Display for Humanized<Duration> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Display::fmt(&Humanized(TimeDelta::from(self.0)), f)
        }
    }
    impl Display for Humanized<Time> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Display::fmt(&Humanized(self.0 - Time::ZERO), f)
        }
    }

    #[derive(Debug, Clone, Error)]
    pub enum ParseTimeDeltaError {
        #[error("Expected a sequence of numbers and units")]
        UnrecognizedTerm,
        #[error(transparent)]
        FloatConversionError(#[from] ParseFloatError),
    }
    impl FromStr for Humanized<TimeDelta> {
        type Err = ParseTimeDeltaError;

        fn from_str(mut s: &str) -> Result<Self, Self::Err> {
            s = s.trim();
            /* let ZERO_RE = lazy_regex!(
                r"^(?:(?:(?:-?(?:\d+(?:\.\d*)?|\.\d+)(?:e\d+)?)\s*(?:y|d|h|m|s)\s*)+|(?:))$"gmi
            ); */
            if regex_is_match!(r"^-?(?:0+(?:\.0*)?|\.0+)(?:e\d+)?$"i, s) {
                return Ok(Humanized(TimeDelta::ZERO));
            }

            let mut total = TimeDelta::ZERO;
            while !s.is_empty() {
                let Some((_, num, unit, rest)) = regex_captures!(
                    r"^(-?(?:\d+(?:\.\d*)?|\.\d+)(?:e\d+)?)\s*([ydhms])\s*([^\s].*)?$",
                    s
                ) else {
                    return Err(ParseTimeDeltaError::UnrecognizedTerm);
                };

                let unit = match unit {
                    "y" => TimeDelta::YEAR,
                    "d" => TimeDelta::DAY,
                    "h" => TimeDelta::HOUR,
                    "m" => TimeDelta::MINUTE,
                    "s" => TimeDelta::SECOND,
                    _ => unreachable!(),
                };

                total += if let Ok(num) = num.parse::<i64>() {
                    num * unit
                } else {
                    let num = num.parse::<f64>()?;
                    num * unit
                };

                s = rest
            }

            Ok(Humanized(total))
        }
    }

    #[derive(Debug, Clone, Error)]
    pub enum ParseDurationError {
        #[error(transparent)]
        NegativeTimeDelta(#[from] NegativeTimeDelta),
        #[error(transparent)]
        ParseTimeDeltaError(#[from] ParseTimeDeltaError),
    }

    impl FromStr for Humanized<Duration> {
        type Err = ParseDurationError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let delta = s.parse::<Humanized<TimeDelta>>()?.0;
            let duration = delta.try_into()?;
            Ok(Humanized(duration))
        }
    }
    impl FromStr for Humanized<Time> {
        type Err = ParseDurationError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let delta = s.parse::<Humanized<TimeDelta>>()?.0;
            let time = Time::ZERO + delta;
            Ok(Humanized(time))
        }
    }

    impl<T> SerializeAs<T> for Humanized<T>
    where
        T: Copy,
        Humanized<T>: Display,
    {
        fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_str(&Humanized(*source))
        }
    }
    impl<'de, T> DeserializeAs<'de, T> for Humanized<T>
    where
        Humanized<T>: FromStr + Into<T>,
        <Humanized<T> as FromStr>::Err: Display,
    {
        fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = <&'de str>::deserialize(deserializer)?;
            s.parse::<Humanized<T>>()
                .map(Into::into)
                .map_err(D::Error::custom)
        }
    }
    pub type HumanizeIfNeeded<T> = IfIsHumanReadable<Humanized<T>>;

    #[cfg(test)]
    mod tests {

        /// checks that displayed values are parsed to the same values
        mod parse_displayed {
            use arbtest::arbtest;

            use crate::{humanized::Humanized, Duration, Time, TimeDelta};

            #[test]
            fn timedelta() {
                arbtest(|u| {
                    let arb: TimeDelta = u.arbitrary()?;
                    let displayed = format!("{}", Humanized(arb));
                    let parsed = displayed.parse::<Humanized<TimeDelta>>().unwrap().0;

                    assert_eq!(arb, parsed);
                    Ok(())
                });
            }

            #[test]
            fn time() {
                arbtest(|u| {
                    let arb: Time = u.arbitrary()?;
                    let displayed = format!("{}", Humanized(arb));
                    let parsed = displayed.parse::<Humanized<Time>>().unwrap().0;

                    assert_eq!(arb, parsed);
                    Ok(())
                });
            }

            #[test]
            fn duration() {
                arbtest(|u| {
                    let arb: Duration = u.arbitrary()?;
                    let displayed = format!("{}", Humanized(arb));
                    let parsed = displayed.parse::<Humanized<Duration>>().unwrap().0;

                    assert_eq!(arb, parsed);
                    Ok(())
                });
            }
        }
    }
}
