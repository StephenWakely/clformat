//! Trait and implementations to help us format numbers of different types

pub trait Num: Copy + PartialOrd {
    fn divide_by(self, divisor: isize) -> isize;
    fn zero() -> Self;
}

macro_rules! impl_num {
    ($t:ty) => {
        impl Num for $t {
            fn divide_by(self, divisor: isize) -> isize {
                self as isize / divisor
            }

            fn zero() -> Self {
                0
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

impl_num!(usize);
impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(u128);
