//! Helpers for the `~R` (English / Roman numeral) and `~M` directives.
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

const ONES: [&str; 20] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];

const TENS: [&str; 8] = [
    "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

const SCALES: [&str; 7] = [
    "",
    "thousand",
    "million",
    "billion",
    "trillion",
    "quadrillion",
    "quintillion",
];

/// Format a number between 0 and 999 (inclusive) as English words.
fn under_1000(mut n: u32) -> String {
    let mut parts: Vec<String> = Vec::new();

    if n >= 100 {
        parts.push(format!("{} hundred", ONES[(n / 100) as usize]));
        n %= 100;
    }

    if n >= 20 {
        let tens_word = TENS[(n / 10 - 2) as usize];
        if n.is_multiple_of(10) {
            parts.push(tens_word.to_string());
        } else {
            parts.push(format!("{}-{}", tens_word, ONES[(n % 10) as usize]));
        }
    } else if n > 0 {
        parts.push(ONES[n as usize].to_string());
    }

    parts.join(" ")
}

/// Formats `n` as an English cardinal number, e.g. `42` -> `"forty-two"`.
///
/// Supports magnitudes up to the quintillions; beyond that the scale name is
/// omitted (this is considered a reasonable limitation for a format library).
pub fn to_cardinal(n: i128) -> String {
    if n == 0 {
        return "zero".to_string();
    }

    let neg = n < 0;
    let mut remaining = n.unsigned_abs();

    let mut groups = Vec::new();
    while remaining > 0 {
        groups.push((remaining % 1000) as u32);
        remaining /= 1000;
    }

    let mut parts = Vec::new();
    for (idx, group) in groups.iter().enumerate().rev() {
        if *group == 0 {
            continue;
        }

        let words = under_1000(*group);
        match SCALES.get(idx) {
            Some(scale) if !scale.is_empty() => parts.push(format!("{words} {scale}")),
            _ => parts.push(words),
        }
    }

    let result = parts.join(" ");

    if neg {
        format!("negative {result}")
    } else {
        result
    }
}

/// Converts the final word of an English number phrase into its ordinal
/// form, e.g. `"two"` -> `"second"`, `"forty"` -> `"fortieth"`.
fn ordinal_word(word: &str) -> String {
    match word {
        "zero" => "zeroth",
        "one" => "first",
        "two" => "second",
        "three" => "third",
        "four" => "fourth",
        "five" => "fifth",
        "six" => "sixth",
        "seven" => "seventh",
        "eight" => "eighth",
        "nine" => "ninth",
        "ten" => "tenth",
        "eleven" => "eleventh",
        "twelve" => "twelfth",
        "thirteen" => "thirteenth",
        "fourteen" => "fourteenth",
        "fifteen" => "fifteenth",
        "sixteen" => "sixteenth",
        "seventeen" => "seventeenth",
        "eighteen" => "eighteenth",
        "nineteen" => "nineteenth",
        "twenty" => "twentieth",
        "thirty" => "thirtieth",
        "forty" => "fortieth",
        "fifty" => "fiftieth",
        "sixty" => "sixtieth",
        "seventy" => "seventieth",
        "eighty" => "eightieth",
        "ninety" => "ninetieth",
        "hundred" => "hundredth",
        other => return format!("{other}th"),
    }
    .to_string()
}

/// Applies [`ordinal_word`] to the last hyphenated component of `word`
/// (e.g. `"forty-two"` -> `"forty-second"`).
fn ordinalize_last(word: &str) -> String {
    match word.rfind('-') {
        Some(pos) => {
            let (head, tail) = word.split_at(pos + 1);
            format!("{head}{}", ordinal_word(tail))
        }
        None => ordinal_word(word),
    }
}

/// Formats `n` as an English ordinal number, e.g. `42` -> `"forty-second"`.
pub fn to_ordinal(n: i128) -> String {
    let cardinal = to_cardinal(n);

    match cardinal.rfind(' ') {
        Some(pos) => {
            let (head, last) = cardinal.split_at(pos + 1);
            format!("{head}{}", ordinalize_last(last))
        }
        None => ordinalize_last(&cardinal),
    }
}

const ROMAN_MODERN: &[(u32, &str)] = &[
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

// The "old style" Roman numerals never used subtractive notation, e.g. 4 was
// written "IIII" rather than "IV".
const ROMAN_OLD: &[(u32, &str)] = &[
    (1000, "M"),
    (500, "D"),
    (100, "C"),
    (50, "L"),
    (10, "X"),
    (5, "V"),
    (1, "I"),
];

/// Formats `n` as a Roman numeral. `n` must be between 1 and 3999 inclusive;
/// outside that range the digits that cannot be represented are omitted.
pub fn to_roman(n: i128, old_style: bool) -> String {
    let table = if old_style { ROMAN_OLD } else { ROMAN_MODERN };

    if n <= 0 {
        // Roman numerals have no standard representation for zero or
        // negative numbers.
        return n.to_string();
    }

    let mut n = n as u32;
    let mut out = String::new();

    for &(value, symbol) in table {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cardinal_numbers() {
        assert_eq!("zero", to_cardinal(0));
        assert_eq!("four", to_cardinal(4));
        assert_eq!("forty-two", to_cardinal(42));
        assert_eq!("one hundred", to_cardinal(100));
        assert_eq!("one hundred two", to_cardinal(102));
        assert_eq!("negative five", to_cardinal(-5));
        assert_eq!(
            "one million two hundred thirty-four thousand five hundred sixty-seven",
            to_cardinal(1_234_567)
        );
    }

    #[test]
    fn ordinal_numbers() {
        assert_eq!("fourth", to_ordinal(4));
        assert_eq!("forty-second", to_ordinal(42));
        assert_eq!("one hundredth", to_ordinal(100));
        assert_eq!("twenty-first", to_ordinal(21));
    }

    #[test]
    fn roman_numerals() {
        assert_eq!("IV", to_roman(4, false));
        assert_eq!("MCMXCIX", to_roman(1999, false));
        assert_eq!("IIII", to_roman(4, true));
        assert_eq!("VIIII", to_roman(9, true));
    }
}
