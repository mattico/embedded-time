//! An instant of time

use crate::{
    duration::{self, Duration},
    fixed_point::FixedPoint,
};
use core::{cmp::Ordering, convert::TryFrom, ops};
use num::traits::{WrappingAdd, WrappingSub};

/// Represents an instant of time relative to a specific [`Clock`](clock/trait.Clock.html)
///
/// # Example
///
/// Typically an `Instant` will be obtained from a [`Clock`](clock/trait.Clock.html)
///
/// ```rust
/// # use embedded_time::{fraction::Fraction, Instant, Clock as _};
/// # #[derive(Debug)]
/// # struct SomeClock;
/// # impl embedded_time::Clock for SomeClock {
/// #     type T = u32;
/// #     type ImplError = ();
/// #     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
/// #     fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {Ok(Instant::<Self>::new(23))}
/// # }
/// let some_clock = SomeClock;
/// let some_instant = some_clock.try_now().unwrap();
/// ```
///
/// However, an `Instant` can also be constructed directly. In this case the constructed `Instant`
/// is `23 * SomeClock::SCALING_FACTOR` seconds since the clock's epoch
///
/// ```rust,no_run
/// # use embedded_time::{fraction::Fraction, Instant};
/// # #[derive(Debug)]
/// # struct SomeClock;
/// # impl embedded_time::Clock for SomeClock {
/// #     type T = u32;
/// #     type ImplError = ();
/// #     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
/// #     fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
/// # }
/// Instant::<SomeClock>::new(23);
/// ```
#[derive(Debug)]
pub struct Instant<Clock: crate::Clock> {
    ticks: Clock::T,
}

impl<Clock: crate::Clock> Instant<Clock> {
    /// Construct a new Instant from the provided [`Clock`](clock/trait.Clock.html)
    pub fn new(ticks: Clock::T) -> Self {
        Self { ticks }
    }

    /// Returns the amount of time elapsed from another instant to this one, or None if that instant
    /// is later than this one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use embedded_time::{Clock as _, duration::*, fraction::Fraction, Instant, ConversionError};
    /// # use core::convert::TryInto;
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// #   type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    ///
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert_eq!(Instant::<Clock>::new(5).checked_duration_since(&Instant::<Clock>::new(3)).unwrap().try_into(),
    ///     Ok(Microseconds(2_000_u64)));
    ///
    /// assert_eq!(Instant::<Clock>::new(3).checked_duration_since(&Instant::<Clock>::new(5)), None);
    /// ```
    pub fn checked_duration_since(&self, other: &Self) -> Option<duration::Generic<Clock::T>> {
        if self >= other {
            Some(duration::Generic::new(
                self.ticks.wrapping_sub(&other.ticks),
                Clock::SCALING_FACTOR,
            ))
        } else {
            None
        }
    }

    /// Returns the amount of time from this instant to another, or None if that instant is earlier
    /// than this one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant, ConversionError};
    /// # use core::convert::TryInto;
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert_eq!(Instant::<Clock>::new(5).checked_duration_until(&Instant::<Clock>::new(7)).unwrap().try_into(),
    ///     Ok(Microseconds(2_000_u64)));
    ///
    /// assert_eq!(Instant::<Clock>::new(7).checked_duration_until(&Instant::<Clock>::new(5)),
    ///     None);
    /// ```
    pub fn checked_duration_until(&self, other: &Self) -> Option<duration::Generic<Clock::T>> {
        if self <= other {
            Some(duration::Generic::new(
                other.ticks.wrapping_sub(&self.ticks),
                Clock::SCALING_FACTOR,
            ))
        } else {
            None
        }
    }

    /// Returns the [`Duration`] (in the provided units) since the beginning of time (the
    /// [`Clock`](clock/trait.Clock.html)'s 0)
    ///
    /// If it is a _wrapping_ clock, the result is meaningless.
    pub fn duration_since_epoch(&self) -> duration::Generic<Clock::T> {
        duration::Generic::new(self.ticks, Clock::SCALING_FACTOR)
    }

    /// This `Instant` + [`Duration`] = later (future) `Instant`
    ///
    /// Returns [`None`] if the [`Duration`] is too large
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant, ConversionError};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert_eq!(
    ///     Instant::<Clock>::new(0).checked_add(Milliseconds(u32::MAX/2)),
    ///     Some(Instant::<Clock>::new(u32::MAX/2))
    /// );
    ///
    /// assert_eq!(
    ///     Instant::<Clock>::new(0).checked_add(Milliseconds(u32::MAX/2 + 1)),
    ///     None
    /// );
    /// ```
    pub fn checked_add<Dur: Duration>(self, duration: Dur) -> Option<Self>
    where
        Dur: FixedPoint,
        Clock::T: TryFrom<Dur::T>,
    {
        let add_ticks: Clock::T = duration.into_ticks(Clock::SCALING_FACTOR).ok()?;
        if add_ticks <= (<Clock::T as num::Bounded>::max_value() / 2.into()) {
            Some(Self {
                ticks: self.ticks.wrapping_add(&add_ticks),
            })
        } else {
            None
        }
    }

    /// This `Instant` - [`Duration`] = earlier `Instant`
    ///
    /// Returns [`None`] if the [`Duration`] is too large
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant, ConversionError};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert_eq!(Instant::<Clock>::new(u32::MAX).checked_sub(Milliseconds(u32::MAX/2)),
    ///     Some(Instant::<Clock>::new(u32::MAX - u32::MAX/2)));
    ///
    /// assert_eq!(Instant::<Clock>::new(u32::MAX).checked_sub(Milliseconds(u32::MAX/2 + 1)),
    ///     None);
    /// ```
    pub fn checked_sub<Dur: Duration>(self, duration: Dur) -> Option<Self>
    where
        Dur: FixedPoint,
        Clock::T: TryFrom<Dur::T>,
    {
        let sub_ticks: Clock::T = duration.into_ticks(Clock::SCALING_FACTOR).ok()?;
        if sub_ticks <= (<Clock::T as num::Bounded>::max_value() / 2.into()) {
            Some(Self {
                ticks: self.ticks.wrapping_sub(&sub_ticks),
            })
        } else {
            None
        }
    }
}

impl<Clock: crate::Clock> Copy for Instant<Clock> {}

impl<Clock: crate::Clock> Clone for Instant<Clock> {
    fn clone(&self) -> Self {
        Self { ticks: self.ticks }
    }
}

impl<Clock: crate::Clock> PartialEq for Instant<Clock> {
    fn eq(&self, other: &Self) -> bool {
        self.ticks == other.ticks
    }
}

impl<Clock: crate::Clock> Eq for Instant<Clock> {}

impl<Clock: crate::Clock> PartialOrd for Instant<Clock> {
    /// Calculates the difference between two `Instant`s resulting in a [`Duration`]
    ///
    /// ```rust
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert!(Instant::<Clock>::new(5) > Instant::<Clock>::new(3));
    /// assert!(Instant::<Clock>::new(5) == Instant::<Clock>::new(5));
    /// assert!(Instant::<Clock>::new(u32::MAX) < Instant::<Clock>::new(u32::MIN));
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<Clock: crate::Clock> Ord for Instant<Clock> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticks
            .wrapping_sub(&other.ticks)
            .cmp(&(<Clock::T as num::Bounded>::max_value() / 2.into()))
            .reverse()
    }
}

impl<Clock: crate::Clock, Dur: Duration> ops::Add<Dur> for Instant<Clock>
where
    Clock::T: TryFrom<Dur::T>,
    Dur: FixedPoint,
{
    type Output = Self;

    /// Add a [`Duration`] to an `Instant` resulting in a new, later `Instant`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert_eq!(Instant::<Clock>::new(1) + Seconds(3_u32),
    ///     Instant::<Clock>::new(3_001));
    /// assert_eq!(Instant::<Clock>::new(1) + Milliseconds(700_u32),
    ///     Instant::<Clock>::new(701));
    /// assert_eq!(Instant::<Clock>::new(1) + Milliseconds(700_u64),
    ///     Instant::<Clock>::new(701));
    ///
    /// // maximum duration allowed
    /// assert_eq!(Instant::<Clock>::new(0) + Milliseconds(i32::MAX as u32),
    ///    Instant::<Clock>::new(u32::MAX/2));
    /// ```
    ///
    /// # Panics
    ///
    /// Virtually the same reason the integer operation would panic. Namely, if the
    /// result overflows the type. Specifically, if the duration is more than half
    /// the wrap-around period of the clock.
    ///
    /// ```rust,should_panic
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// Instant::<Clock>::new(0) + Milliseconds(u32::MAX/2 + 1);
    /// ```
    fn add(self, rhs: Dur) -> Self::Output {
        self.checked_add(rhs).unwrap()
    }
}

impl<Clock: crate::Clock, Dur: Duration> ops::Sub<Dur> for Instant<Clock>
where
    Clock::T: TryFrom<Dur::T>,
    Dur: FixedPoint,
{
    type Output = Self;

    /// Subtract a [`Duration`] from an `Instant` resulting in a new, earlier `Instant`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// assert_eq!(Instant::<Clock>::new(5_001) - Seconds(3_u32),
    ///     Instant::<Clock>::new(2_001));
    /// assert_eq!(Instant::<Clock>::new(800) - Milliseconds(700_u32),
    ///     Instant::<Clock>::new(100));
    /// assert_eq!(Instant::<Clock>::new(5_000) - Milliseconds(700_u64),
    ///     Instant::<Clock>::new(4_300));
    ///
    /// // maximum duration allowed
    /// assert_eq!(Instant::<Clock>::new(u32::MAX) - Milliseconds(i32::MAX as u32),
    ///     Instant::<Clock>::new(u32::MAX/2 + 1));
    /// ```
    ///
    /// # Panics
    ///
    /// Virtually the same reason the integer operation would panic. Namely, if the
    /// result overflows the type. Specifically, if the duration is more than half
    /// the wrap-around period of the clock.
    ///
    /// ```rust,should_panic
    /// # use embedded_time::{fraction::Fraction, duration::*, Instant};
    /// # #[derive(Debug)]
    /// struct Clock;
    /// impl embedded_time::Clock for Clock {
    ///     type T = u32;
    /// # type ImplError = ();
    ///     const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);
    ///     // ...
    /// # fn try_now(&self) -> Result<Instant<Self>, embedded_time::clock::Error<Self::ImplError>> {unimplemented!()}
    /// }
    ///
    /// Instant::<Clock>::new(u32::MAX) - Milliseconds(u32::MAX/2 + 1);
    /// ```
    fn sub(self, rhs: Dur) -> Self::Output {
        self.checked_sub(rhs).unwrap()
    }
}

#[cfg(test)]
mod tests {}
