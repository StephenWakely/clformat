#![no_std]
extern crate alloc;

pub use clformat_macro::clformat;

mod chars;
mod decimal;
mod english;
mod exponential;
#[cfg(feature = "std")]
mod io_adapter;
mod num;
mod ruler;
mod tracker;

pub use chars::char_name;
pub use decimal::Decimal;
pub use english::{to_cardinal, to_ordinal, to_roman};
pub use exponential::{format_exponential, general_params};
#[cfg(feature = "std")]
pub use io_adapter::IoWriteAdapter;
pub use ruler::Ruler;
pub use tracker::ColumnTracker;
