//! Trait and implementations to help us format numbers of different types

pub trait Num: Copy + PartialOrd {
    fn divide_by(self, divisor: isize) -> isize;
    fn multiply_by(self, factor: isize) -> Self;
    fn subtract_by(self, num: isize) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn as_u8(self) -> u8;
    fn as_usize(self) -> usize;
}

macro_rules! impl_num {
    ($t:ty) => {
        impl Num for $t {
            fn divide_by(self, divisor: isize) -> isize {
                self as isize / divisor
            }

            fn multiply_by(self, factor: isize) -> Self {
                self * factor as Self
            }

            fn subtract_by(self, num: isize) -> Self {
                self - num as Self
            }

            fn as_u8(self) -> u8 {
                self as u8
            }

            fn as_usize(self) -> usize {
                self as usize
            }

            fn zero() -> Self {
                0 as Self
            }

            fn one() -> Self {
                1 as Self
            }
        }
    };
}

impl_num!(isize);
impl_num!(i8);
impl_num!(i16);
impl_num!(i32);
impl_num!(i64);
impl_num!(i128);

impl_num!(f32);
impl_num!(f64);

impl_num!(usize);
impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(u128);
