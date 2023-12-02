#![no_std]
pub use clformat_macro::clformat;

mod decimal;
mod num;
mod ruler;

pub use decimal::Decimal;
pub use ruler::Ruler;
