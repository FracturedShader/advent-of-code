/// Like [[`num_traits::Zero`]] and [[`num_traits::One`]], but for Two
pub trait Two {
    #[allow(unused)]
    fn two() -> Self;
}

/// Like [[`num_traits::ConstZero`]] and [[`num_traits::ConstOne`]], but for Two
pub trait ConstTwo: Two {
    const TWO: Self;
}

macro_rules! two_impl {
    ($t:ty, $v:literal) => {
        impl Two for $t {
            #[inline]
            fn two() -> $t {
                $v
            }
        }

        impl ConstTwo for $t {
            const TWO: Self = $v;
        }
    };
}

two_impl!(usize, 2);
two_impl!(u8, 2);
two_impl!(u16, 2);
two_impl!(u32, 2);
two_impl!(u64, 2);
two_impl!(u128, 2);

two_impl!(isize, 2);
two_impl!(i8, 2);
two_impl!(i16, 2);
two_impl!(i32, 2);
two_impl!(i64, 2);
two_impl!(i128, 2);

two_impl!(f32, 2.0);
two_impl!(f64, 2.0);

/// When using a numeric type to represent the extents of a box in a subdividing hierarchy, there
/// are various limits that differ depending on the underlying type. This trait represents a way to
/// determine those limits and make necessary adjustments to a value.
pub trait ExtentNum: Copy {
    /// Can the current number be subdivided and still have meaning as the extents of a box?
    ///
    /// - For integers this means `self >= 2`
    /// - For floating-point numbers this means `self >= 2.0 * Self::MIN_POSITIVE`
    fn can_subdivide(self) -> bool;

    /// Essentially: is the type limited to integers? Extents are always divided by two when
    /// subdivided. Since the subdivisions must fully cover the same space in a non-overlapping
    /// manner, this requires integers to be powers of two.
    #[allow(unused)]
    fn must_be_power_of_two() -> bool;

    /// If [[`ExtentNum::must_be_power_of_two`]], this aims to return the power of two at least as
    /// large as the current value.
    fn to_power_of_two(self) -> Option<Self>;
}

/// Extends the base [[`ExtentNum`]] trait to support `const` answers to questions where possible.
/// Every numeric type should be able to implement this.
pub trait ConstExtentNum: ExtentNum {
    /// Essentially: is the type limited to integers? Extents are always divided by two when
    /// subdivided. Since the subdivisions must fully cover the same space in a non-overlapping
    /// manner, this requires integers to be powers of two.
    const MUST_BE_POWER_OF_TWO: bool;
}

macro_rules! extent_uint_impl {
    ($t:ty) => {
        impl ExtentNum for $t {
            #[inline]
            fn can_subdivide(self) -> bool {
                self >= 2
            }

            #[inline]
            fn must_be_power_of_two() -> bool {
                true
            }

            #[inline]
            fn to_power_of_two(self) -> Option<Self> {
                if self.is_power_of_two() {
                    Some(self)
                } else {
                    Some(self.next_power_of_two())
                }
            }
        }

        impl ConstExtentNum for $t {
            const MUST_BE_POWER_OF_TWO: bool = true;
        }
    };
}

macro_rules! extent_int_impl {
    ($t:ty, $ut:ty) => {
        impl ExtentNum for $t {
            #[inline]
            fn can_subdivide(self) -> bool {
                self >= 2
            }

            #[inline]
            fn must_be_power_of_two() -> bool {
                true
            }

            #[inline]
            fn to_power_of_two(self) -> Option<Self> {
                <$ut>::try_from(self)
                    .ok()
                    .and_then(<$ut>::to_power_of_two)
                    .and_then(|i| i.try_into().ok())
            }
        }

        impl ConstExtentNum for $t {
            const MUST_BE_POWER_OF_TWO: bool = true;
        }
    };
}

macro_rules! extent_float_impl {
    ($t:ty) => {
        impl ExtentNum for $t {
            #[inline]
            fn can_subdivide(self) -> bool {
                self >= 2.0 * Self::MIN_POSITIVE
            }

            #[inline]
            fn must_be_power_of_two() -> bool {
                false
            }

            #[inline]
            fn to_power_of_two(self) -> Option<Self> {
                None
            }
        }

        impl ConstExtentNum for $t {
            const MUST_BE_POWER_OF_TWO: bool = false;
        }
    };
}

extent_uint_impl!(usize);
extent_uint_impl!(u8);
extent_uint_impl!(u16);
extent_uint_impl!(u32);
extent_uint_impl!(u64);
extent_uint_impl!(u128);

extent_int_impl!(isize, usize);
extent_int_impl!(i8, u8);
extent_int_impl!(i16, u16);
extent_int_impl!(i32, u32);
extent_int_impl!(i64, u64);
extent_int_impl!(i128, u128);

extent_float_impl!(f32);
extent_float_impl!(f64);

pub trait OctreeNum:
    Copy
    + Default
    + Eq
    + PartialOrd
    + num_traits::Zero
    + nalgebra::ClosedAddAssign
    + nalgebra::ClosedDivAssign
    + nalgebra::ClosedMulAssign
    + nalgebra::ClosedSubAssign
    + nalgebra::Scalar
    + nalgebra::SimdPartialOrd
    + ConstExtentNum
    + ConstTwo
{
}

impl<N> OctreeNum for N where
    N: Copy
        + Default
        + Eq
        + PartialOrd
        + num_traits::Zero
        + nalgebra::ClosedAddAssign
        + nalgebra::ClosedDivAssign
        + nalgebra::ClosedMulAssign
        + nalgebra::ClosedSubAssign
        + nalgebra::Scalar
        + nalgebra::SimdPartialOrd
        + ConstExtentNum
        + ConstTwo
{
}
