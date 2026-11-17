//! Decimal helper struct to format decimals (and, more generally, any radix
//! between 2 and 36 - used to implement `~D`, `~X`, `~O`, `~B` and `~R`).
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
    radix: usize,
}

fn divisor<T: Num>(number: T, radix: usize) -> (usize, usize) {
    let mut divisor: isize = 1;
    let mut count = 1;
    while number.divide_by(divisor).abs() >= radix as isize {
        divisor *= radix as isize;
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
        Self::new_radix(
            10,
            min_columns,
            pad_char,
            comma_char,
            comma_interval,
            print_commas,
            print_sign,
            number,
        )
    }

    /// Like [`Decimal::new`] but allows the radix (base) to be specified.
    /// Used to implement `~X` (16), `~O` (8), `~B` (2) and `~R` with an
    /// explicit radix parameter.
    #[allow(clippy::too_many_arguments)]
    pub fn new_radix(
        radix: usize,
        min_columns: usize,
        pad_char: char,
        comma_char: char,
        comma_interval: usize,
        print_commas: bool,
        print_sign: bool,
        number: T,
    ) -> Self {
        let (divisor, digits) = divisor(number, radix);

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
            radix,
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
        let digit = self.number.divide_by(self.divisor as isize) % self.radix as isize;
        self.divisor /= self.radix;
        self.digits -= 1;

        let digit =
            core::char::from_digit(digit.unsigned_abs() as u32, self.radix as u32).unwrap_or('?');

        // Use uppercase letters for digits above 9 (e.g. hexadecimal), to
        // match the convention used by Common Lisp implementations.
        Some(if self.radix > 10 {
            digit.to_ascii_uppercase()
        } else {
            digit
        })
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
    fn prints_other_radixes() {
        let decimal = Decimal::new_radix(16, 0, ' ', ',', 3, false, false, 255);
        assert_eq!("FF".to_string(), decimal.collect::<String>());

        let decimal = Decimal::new_radix(8, 0, ' ', ',', 3, false, false, 8);
        assert_eq!("10".to_string(), decimal.collect::<String>());

        let decimal = Decimal::new_radix(2, 0, ' ', ',', 3, false, false, 10);
        assert_eq!("1010".to_string(), decimal.collect::<String>());

        let decimal = Decimal::new_radix(16, 4, '0', ',', 3, false, false, 255);
        assert_eq!("00FF".to_string(), decimal.collect::<String>());

        let decimal = Decimal::new_radix(16, 0, ' ', ',', 3, false, false, -255);
        assert_eq!("-FF".to_string(), decimal.collect::<String>());
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
