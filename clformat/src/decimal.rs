//! Decimal helper struct to format decimals.
use crate::num::Num;

#[derive(Clone, Debug, Default)]
pub struct Decimal<T> {
    number: T,
    pad_char: char,
    comma_char: char,
    comma_interval: usize,
    divisor: usize,
    digits: usize,
    print_commas: bool,
    printed_comma: bool,
    print_sign: bool,
    printed_sign: bool,
    pad: usize,
}

fn divisor<T: Num>(number: T) -> (usize, usize) {
    let mut divisor = 1;
    let mut count = 1;
    while number.divide_by(divisor).abs() >= 10 {
        divisor *= 10;
        count += 1;
    }

    (divisor as usize, count)
}

impl<T: Num> Decimal<T> {
    pub fn new(
        min_columns: usize,
        pad_char: char,
        comma_char: char,
        comma_interval: usize,
        print_commas: bool,
        print_sign: bool,
        number: T,
    ) -> Self {
        let (divisor, digits) = divisor(number);

        // Take the sign and any commas into consideration when calculating -
        // the number of columns for padding.
        let columns = if number < T::zero() || print_sign {
            digits + 1
        } else {
            digits
        } + if print_commas {
            (digits - 1) / comma_interval
        } else {
            0
        };

        let pad = if min_columns > digits {
            min_columns - columns
        } else {
            0
        };

        Self {
            pad_char,
            comma_char,
            comma_interval,
            print_commas,
            // Set to true so we don't output a comma at the first char
            printed_comma: true,
            print_sign,
            printed_sign: false,
            number,
            divisor,
            digits,
            pad,
        }
    }
}

impl<T: Num> core::iter::Iterator for Decimal<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.divisor == 0 {
            return None;
        }

        if self.pad > 0 {
            self.pad -= 1;
            return Some(self.pad_char);
        }

        if !self.printed_sign {
            self.printed_sign = true;
            if self.number > T::zero() {
                if self.print_sign {
                    return Some('+');
                }
            } else {
                return Some('-');
            }
        }

        if self.print_commas
            && self.digits % self.comma_interval == 0
            && self.divisor != 1
            && !self.printed_comma
        {
            self.printed_comma = true;
            return Some(self.comma_char);
        }

        self.printed_comma = false;
        let digit = self.number.divide_by(self.divisor as isize) % 10;
        self.divisor /= 10;
        self.digits -= 1;

        Some(core::char::from_digit(digit.unsigned_abs() as u32, 10).unwrap())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::{String, ToString};

    use super::*;

    #[test]
    fn prints_commas() {
        let decimal = Decimal::new(0, ' ', ',', 3, true, false, 420);
        let num = decimal.collect::<String>();
        assert_eq!("420".to_string(), num);

        let decimal = Decimal::new(0, ' ', ',', 3, true, false, 4200);
        let num = decimal.collect::<String>();
        assert_eq!("4,200".to_string(), num);

        let decimal = Decimal::new(0, ' ', ',', 3, true, false, 42000);
        let num = decimal.collect::<String>();
        assert_eq!("42,000".to_string(), num);

        let decimal = Decimal::new(0, ' ', ',', 3, true, false, 4_200_000);
        let num = decimal.collect::<String>();
        assert_eq!("4,200,000".to_string(), num);

        let decimal = Decimal::new(0, ' ', ',', 3, true, false, -4_200_000);
        let num = decimal.collect::<String>();
        assert_eq!("-4,200,000".to_string(), num);
    }

    #[test]
    fn prints_alternative_separators() {
        let decimal = Decimal::new(0, ' ', '_', 3, true, false, 4200);
        let num = decimal.collect::<String>();
        assert_eq!("4_200".to_string(), num);

        let decimal = Decimal::new(0, ' ', '_', 2, true, false, 42000);
        let num = decimal.collect::<String>();
        assert_eq!("4_20_00".to_string(), num);

        let decimal = Decimal::new(0, ' ', '_', 4, true, false, 4_200_000);
        let num = decimal.collect::<String>();
        assert_eq!("420_0000".to_string(), num);
    }

    #[test]
    fn pads() {
        let decimal = Decimal::new(2, ' ', ',', 3, true, false, 420);
        let num = decimal.collect::<String>();
        assert_eq!("420".to_string(), num);

        let decimal = Decimal::new(5, ' ', ',', 3, true, false, 420);
        let num = decimal.collect::<String>();
        assert_eq!("  420".to_string(), num);

        let decimal = Decimal::new(5, ' ', ',', 3, true, false, -420);
        let num = decimal.collect::<String>();
        assert_eq!(" -420".to_string(), num);

        let decimal = Decimal::new(8, '-', ',', 3, true, false, 420);
        let num = decimal.collect::<String>();
        assert_eq!("-----420".to_string(), num);

        let decimal = Decimal::new(8, '-', ',', 3, true, false, 4200);
        let num = decimal.collect::<String>();
        assert_eq!("---4,200".to_string(), num);
    }

    #[test]
    fn sign() {
        let decimal = Decimal::new(2, ' ', ',', 3, true, true, 420);
        let num = decimal.collect::<String>();
        assert_eq!("+420".to_string(), num);

        // Print the negative sign even if print sign is false
        let decimal = Decimal::new(2, ' ', ',', 3, true, false, -420);
        let num = decimal.collect::<String>();
        assert_eq!("-420".to_string(), num);

        let decimal = Decimal::new(2, ' ', ',', 3, true, true, -420);
        let num = decimal.collect::<String>();
        assert_eq!("-420".to_string(), num);
    }
}
