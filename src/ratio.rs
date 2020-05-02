#[allow(unused_imports)]
use core::convert::{TryFrom, TryInto};
use core::{cmp, ops};

/// A simple rational number
#[derive(Copy, Clone, Debug)]
pub struct Ratio<T>
where
    T: Copy,
{
    numerator: T,
    denominator: T,
}

impl<T> Ratio<T>
where
    T: Copy + ops::Div<Output = T>,
{
    pub const fn new(numerator: T, denominator: T) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl<T: cmp::PartialEq + Copy> cmp::PartialEq for Ratio<T> {
    fn eq(&self, other: &Self) -> bool {
        self.numerator == other.numerator && self.denominator == other.denominator
    }
}

macro_rules! int_mul_div_ratio {
    ($($type:ty),+) => {
        $(
            impl ops::Mul<Ratio<$type>> for $type
            {
                type Output = Self;

                fn mul(self, rhs: Ratio<$type>) -> Self::Output {
                    self * rhs.numerator / rhs.denominator
                }
            }

            impl ops::MulAssign<Ratio<$type>> for $type
            {
                fn mul_assign(&mut self, rhs: Ratio<$type>) {
                    *self = *self * rhs.numerator / rhs.denominator
                }
            }


            impl ops::Div<Ratio<$type>> for $type
            {
                type Output = Self;

                fn div(self, rhs: Ratio<$type>) -> Self::Output {
                    self * rhs.denominator / rhs.numerator
                }
            }

            impl ops::DivAssign<Ratio<$type>> for $type
            {
                fn div_assign(&mut self, rhs: Ratio<$type>) {
                    *self = *self * rhs.denominator / rhs.numerator
                }
            }
        )+
    };
}
int_mul_div_ratio![i8, i16, i32, i64, u8, u16, u32, u64];

macro_rules! test_mul_div {
    ($($type:ty),+) => {
        #[cfg(test)]
        mod tests {
            use super::*;

            #[allow(dead_code)]
            fn mul<T>(input: T, output: T)
            where
                T: Copy + ops::Mul<Output = T> + ops::Div<Output = T> + ops::Mul<Ratio<T>, Output = T>+ cmp::PartialEq + core::fmt::Debug + TryFrom<i32>,
                <T as std::convert::TryFrom<i32>>::Error: std::fmt::Debug,
            {
                assert_eq!(input * Ratio::new(T::try_from(1).unwrap(), T::try_from(2).unwrap()), output);
            }

            #[test]
            fn mul_tests() {
                $(
                    mul::<$type>(<$type>::MAX,<$type>::MAX / 2);
                )+
            }

            #[allow(dead_code)]
            fn mul_assign<T>(mut input: T, output: T)
            where
                T: Copy + ops::MulAssign + ops::MulAssign<Ratio<T>> + ops::Mul<Output = T> + ops::Div<Output = T> + ops::Mul<Ratio<T>, Output = T>+ cmp::PartialEq + core::fmt::Debug + TryFrom<i32>,
                <T as std::convert::TryFrom<i32>>::Error: std::fmt::Debug,
            {
                input *= Ratio::new(T::try_from(1).unwrap(), T::try_from(2).unwrap());
                assert_eq!(input, output);
            }

            #[test]
            fn mul_assign_tests() {
                $(
                    mul_assign::<$type>(<$type>::MAX,<$type>::MAX / 2);
                )+
            }

            #[allow(dead_code)]
            fn div<T>(input: T, output: T)
            where
                T: Copy + ops::Div<Output = T> + ops::Div<Ratio<T>, Output = T>+ cmp::PartialEq + core::fmt::Debug + TryFrom<i32>,
                <T as std::convert::TryFrom<i32>>::Error: std::fmt::Debug,
            {
                assert_eq!(input / Ratio::new(T::try_from(1).unwrap(), T::try_from(2).unwrap()), output);
            }

            #[test]
            fn div_tests() {
                $(
                    div::<$type>((<$type>::MAX - <$type>::MAX % 2) / 2, <$type>::MAX - <$type>::MAX % 2);
                )+
            }

            #[allow(dead_code)]
            fn div_assign<T>(mut input: T, output: T)
            where
                T: Copy + ops::DivAssign + ops::DivAssign<Ratio<T>> + ops::Mul<Output = T> + ops::Div<Output = T> + ops::Mul<Ratio<T>, Output = T>+ cmp::PartialEq + core::fmt::Debug + TryFrom<i32>,
                <T as std::convert::TryFrom<i32>>::Error: std::fmt::Debug,
            {
                input /= Ratio::new(T::try_from(1).unwrap(), T::try_from(2).unwrap());
                assert_eq!(input, output);
            }

            #[test]
            fn div_assign_tests() {
                $(
                    div_assign::<$type>((<$type>::MAX - <$type>::MAX % 2) / 2, <$type>::MAX - <$type>::MAX % 2);
                )+
            }


        }
    };
}
test_mul_div![i8, i16, i32, i64, u8, u16, u32, u64];