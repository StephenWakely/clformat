#![allow(warnings)]
use std::iter::Peekable;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{anychar, digit1},
    combinator::{cut, map, map_res},
    error::Error,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use syn::{token::Token, LitStr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Directive {
    TildeA,
    TildeS,
    TildeD,
    Decimal {
        min_columns: usize,
        pad_char: char,
        comma_char: char,
        comma_interval: usize,
    },
    Break,
    Newline,
    Skip,
    Iteration(Vec<Directive>),
    Literal(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Normal,
    Loop,
}

impl Default for State {
    fn default() -> Self {
        Self::Normal
    }
}

pub fn parse_format_string(
    token: LitStr,
    format_string: &str,
) -> Result<Vec<Directive>, syn::Error> {
    parse_string(format_string)
        .map_err(|err| {
            dbg!(&err);
            syn::Error::new_spanned(token, err.to_string())
        })
        .map(|(_, result)| {
            dbg!(&result);
            result
        })
}

/// http://www.lispworks.com/documentation/lw50/CLHS/Body/22_c.htm
fn parse_string(input: &str) -> IResult<&str, Vec<Directive>> {
    many1(cut(segment))(input)
}

fn segment(input: &str) -> IResult<&str, Directive> {
    alt((literal, iteration, directive))(input)
}

fn literal(input: &str) -> IResult<&str, Directive> {
    map(take_till1(|c| c == '~'), |s: &str| {
        Directive::Literal(s.to_string())
    })(input)
}

fn iteration(input: &str) -> IResult<&str, Directive> {
    let (mut input, _) = tag("~{")(input)?;
    let mut result = Vec::new();

    loop {
        if input.starts_with("~}") {
            return Ok((&input[2..], Directive::Iteration(result)));
        } else if input.is_empty() {
            // No end directive at the end of the string could be regarded as an error,
            // but lets be permissive for now.
            return Ok((&input, Directive::Iteration(result)));
        } else {
            let (new_input, directive) = segment(input)?;
            input = new_input;
            result.push(directive);
        }
    }
}

fn directive(input: &str) -> IResult<&str, Directive> {
    map_res(
        preceded(tag("~"), tuple((params, anychar))),
        |(params, directive)| match directive.to_ascii_uppercase() {
            'A' => Ok(Directive::TildeA),
            'S' => Ok(Directive::TildeS),
            'D' => Ok(Directive::TildeD),
            '%' => Ok(Directive::Newline),
            '*' => Ok(Directive::Skip),
            '^' => {
                // if state != State::Loop {
                //     return Err(syn::Error::new_spanned(
                //         token,
                //         "break directive `^` not inside loop",
                //     ));
                // }
                Ok(Directive::Break)
            }
            directive => {
                Err("duff directive")
                // return Err(syn::Error::new_spanned(
                //     token,
                //     format!("Invalid directive: {directive}"),
                // ))
            }
        },
    )(input)
}

#[derive(Debug, PartialEq, Eq)]
enum Param {
    Char(char),
    Num(isize),
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Params {
    parsed: Vec<Param>,
}

impl Params {
    fn new(parsed: Vec<Param>) -> Self {
        Self { parsed }
    }
}

/// Parses a single parameter either:
/// -  an integer
/// -  or a single character preceeded by a quote (')
fn param(input: &str) -> IResult<&str, Param> {
    alt((
        map(digit1, |nums: &str| {
            Param::Num(nums.parse().expect("numbers should have been parsed"))
        }),
        map(preceded(tag("'"), anychar), Param::Char),
    ))(input)
}

/// Nums are parsed as numbers.
/// Chars are preceeded with a quote `'`.
fn params(input: &str) -> IResult<&str, Params> {
    map(separated_list0(tag(","), param), Params::new)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_literal() {
        assert_eq!(
            ("", Directive::Literal("zork".to_string())),
            literal("zork").unwrap()
        );
        assert_eq!(
            ("~A", Directive::Literal("zork".to_string())),
            literal("zork~A").unwrap()
        );
    }

    #[test]
    fn parses_params() {
        assert_eq!(
            (
                "a",
                Params {
                    parsed: vec![Param::Num(42)]
                }
            ),
            params("42a").unwrap()
        );

        assert_eq!(
            (
                "a",
                Params {
                    parsed: vec![Param::Num(42), Param::Char(' ')]
                }
            ),
            params("42,' a").unwrap()
        );

        assert_eq!(
            (
                "a",
                Params {
                    parsed: vec![Param::Num(42), Param::Char(' '), Param::Num(234)]
                }
            ),
            params("42,' ,234a").unwrap()
        );
    }

    #[test]
    fn parses() {
        let format_string = "Hello, ~A! Value: ~D~%";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string).unwrap();

        assert_eq!(
            vec![
                Directive::Literal("Hello, ".to_string()),
                Directive::TildeA,
                Directive::Literal("! Value: ".to_string()),
                Directive::TildeD,
                Directive::Newline
            ],
            parsed
        );
    }

    #[test]
    fn parses_iteration() {
        let format_string = "Hello, ~{~Anork~A~}~%";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string).unwrap();

        assert_eq!(
            vec![
                Directive::Literal("Hello, ".to_string()),
                Directive::Iteration(vec![
                    Directive::TildeA,
                    Directive::Literal("nork".to_string()),
                    Directive::TildeA,
                ]),
                Directive::Newline
            ],
            parsed
        );
    }
}
