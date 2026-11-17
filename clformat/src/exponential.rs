//! Helpers for the `~E` (exponential) and `~G` (general) float directives.
use alloc::format;
use alloc::string::String;

/// Formats `value` in scientific notation with `digits` digits after the
/// decimal point in the mantissa, using `exponent_char` to separate the
/// mantissa from the (explicitly signed) exponent, e.g.
/// `format_exponential(31415.9, 3, 'e')` -> `"3.142e+4"`.
///
/// This is a reasonable approximation of Common Lisp's `~E` directive; it
/// does not implement the `k` (scale factor) or overflow-character
/// parameters.
pub fn format_exponential(value: f64, digits: usize, exponent_char: char) -> String {
    if value == 0.0 {
        let mantissa = if digits > 0 {
            format!("0.{}", "0".repeat(digits))
        } else {
            "0".into()
        };
        return format!("{mantissa}{exponent_char}+0");
    }

    let formatted = format!("{value:.digits$e}");

    match formatted.find('e') {
        Some(pos) => {
            let (mantissa, exponent) = formatted.split_at(pos);
            let exponent: i64 = exponent[1..].parse().unwrap_or(0);
            format!("{mantissa}{exponent_char}{exponent:+}")
        }
        None => formatted,
    }
}

/// Picks fixed or exponential notation depending on the magnitude of
/// `value`, similar to Common Lisp's `~G` directive (and, informally,
/// C's `%g`). Returns `(use_exponential, decimal_places)`.
///
/// `digits` is the number of significant digits requested (defaulting to 6
/// when `None`, mirroring the common convention for this family of
/// directives).
pub fn general_params(value: f64, digits: Option<usize>) -> (bool, usize) {
    let digits = digits.unwrap_or(6).max(1);

    if value == 0.0 {
        return (false, digits.saturating_sub(1));
    }

    let exponent = value.abs().log10().floor() as i64;

    if exponent < -4 || exponent >= digits as i64 {
        (true, digits.saturating_sub(1))
    } else {
        let decimal_places = (digits as i64 - 1 - exponent).max(0) as usize;
        (false, decimal_places)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_exponential() {
        assert_eq!("3.142e+4", format_exponential(31415.9, 3, 'e'));
        assert_eq!("1.000e+0", format_exponential(1.0, 3, 'e'));
        assert_eq!("-1.500e-2", format_exponential(-0.015, 3, 'e'));
    }

    #[test]
    fn formats_exponential_zero() {
        assert_eq!("0.000e+0", format_exponential(0.0, 3, 'e'));
    }

    #[test]
    fn general_chooses_fixed_for_everyday_magnitudes() {
        let (exponential, places) = general_params(123.456, Some(6));
        assert!(!exponential);
        assert_eq!(3, places);
    }

    #[test]
    fn general_chooses_exponential_for_large_magnitudes() {
        let (exponential, _) = general_params(123_456_789.0, Some(6));
        assert!(exponential);
    }

    #[test]
    fn general_chooses_exponential_for_tiny_magnitudes() {
        let (exponential, _) = general_params(0.0000001234, Some(6));
        assert!(exponential);
    }
}
