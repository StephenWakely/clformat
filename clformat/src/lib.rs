#![no_std]
pub use clformat_macro::clformat;

#[derive(Clone, Debug)]
pub struct Decimal {
    pub min_columns: usize,
    pub pad_char: char,
    pub comma_char: char,
    pub comma_interval: usize,
}
