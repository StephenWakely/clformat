#![allow(warnings)]
use std::{fmt::Write, io::Write as _, iter::Peekable, ops::Deref};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_while},
    character::complete::{anychar, digit1},
    combinator::{cut, eof, map, map_res},
    error::FromExternalError,
    multi::{many0, many1, many_till, separated_list0},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use proc_macro2::Span;
use syn::{token::Token, LitStr};

use crate::parse_error::FormatError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Directive {
    Align {
        min_columns: usize,
        col_inc: usize,
        min_pad: usize,
        pad_char: char,
        inner: Vec<Directive>,
    },
    TildeA,
    TildeS,
    Decimal {
        min_columns: usize,
        pad_char: char,
        comma_char: char,
        comma_interval: usize,
        print_commas: bool,
        print_sign: bool,
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
            let err = match err {
                nom::Err::Incomplete(_) => unreachable!(),
                nom::Err::Error(err) => err,
                nom::Err::Failure(err) => err,
            };
            syn::Error::new_spanned(token, err.to_string())
        })
        .map(|(_, result)| result)
}

type FormatResult<'a, T> = IResult<&'a str, T, FormatError<&'a str>>;

/// http://www.lispworks.com/documentation/lw50/CLHS/Body/22_c.htm
fn parse_string(input: &str) -> FormatResult<Vec<Directive>> {
    map(
        many_till(cut(segment(State::Normal)), eof),
        |(directives, _)| {
            // Ignore the eof parser result.
            directives
        },
    )(input)
}

fn segment(state: State) -> impl Fn(&str) -> FormatResult<Directive> {
    move |input| alt((literal, alignment, iteration, directive(state)))(input)
}

fn literal(input: &str) -> FormatResult<Directive> {
    map(take_till1(|c| c == '~'), |s: &str| {
        Directive::Literal(s.to_string())
    })(input)
}

fn params_to_align(params: Params, inner: Vec<Directive>) -> Result<Directive, String> {
    let min_columns = params.get_num(0, 0)? as usize;
    let col_inc = params.get_num(1, 0)? as usize;
    let min_pad = params.get_num(2, 0)? as usize;
    let pad_char = params.get_char(3, ' ')?;

    Ok((Directive::Align {
        min_columns,
        col_inc,
        min_pad,
        pad_char,
        inner,
    }))
}

fn alignment(input: &str) -> FormatResult<Directive> {
    let (input, _) = tag("~")(input)?;
    let (input, params) = params(input)?;
    let (input, modifiers) = modifiers(input)?;
    let (mut input, _) = tag("<")(input)?;

    let mut result = Vec::new();

    loop {
        if input.starts_with("~>") {
            return Ok((
                &input[2..],
                params_to_align(params, result).map_err(|err| {
                    nom::Err::Error(FormatError::from_external_error(
                        input,
                        nom::error::ErrorKind::Tag,
                        err,
                    ))
                })?,
            ));
        } else if input.is_empty() {
            // No end directive at the end of the string could be regarded as an error,
            // but lets be permissive for now.
            return Ok((
                &input,
                params_to_align(params, result).map_err(|err| {
                    nom::Err::Error(FormatError::from_external_error(
                        input,
                        nom::error::ErrorKind::Tag,
                        err,
                    ))
                })?,
            ));
        } else {
            let (new_input, directive) = segment(State::Loop)(input)?;
            input = new_input;
            result.push(directive);
        }
    }
}

fn iteration(input: &str) -> FormatResult<Directive> {
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
            let (new_input, directive) = segment(State::Loop)(input)?;
            input = new_input;
            result.push(directive);
        }
    }
}

fn directive(state: State) -> impl Fn(&str) -> FormatResult<Directive> {
    move |input| {
        map_res(
            preceded(tag("~"), tuple((params, modifiers, anychar))),
            |(params, modifiers, directive)| match directive.to_ascii_uppercase() {
                'A' => Ok(Directive::TildeA),
                'S' => Ok(Directive::TildeS),
                'D' => {
                    let min_columns = params.get_num(0, 0)? as usize;
                    let pad_char = params.get_char(1, ' ')?;
                    let comma_char = params.get_char(2, ',')?;
                    let comma_interval = params.get_num(3, 3)? as usize;

                    Ok(Directive::Decimal {
                        min_columns,
                        pad_char,
                        comma_char,
                        comma_interval,
                        print_commas: modifiers.colon,
                        print_sign: modifiers.at,
                    })
                }
                '%' => Ok(Directive::Newline),
                '*' => Ok(Directive::Skip),
                '^' => {
                    if state != State::Loop {
                        Err("directive `^` not inside loop".to_string())
                    } else {
                        Ok(Directive::Break)
                    }
                }
                directive => Err(format!("invalid directive `~{directive}`")),
            },
        )(input)
    }
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

    pub fn get_num(&self, idx: usize, def: isize) -> Result<isize, String> {
        match self.parsed.get(idx) {
            Some(Param::Char(c)) => Err(format!("expected number, found char {c}")),
            Some(Param::Num(i)) => Ok(*i),
            None => Ok(def),
        }
    }

    pub fn get_char(&self, idx: usize, def: char) -> Result<char, String> {
        match self.parsed.get(idx) {
            Some(Param::Num(i)) => Err(format!("expected character, found number {i}")),
            Some(Param::Char(c)) => Ok(*c),
            None => Ok(def),
        }
    }
}

/// Parses a single parameter either:
/// -  an integer
/// -  or a single character preceeded by a quote (')
fn param(input: &str) -> FormatResult<Param> {
    alt((
        map(digit1, |nums: &str| {
            Param::Num(nums.parse().expect("numbers should have been parsed"))
        }),
        map(preceded(tag("'"), anychar), Param::Char),
    ))(input)
}

/// Nums are parsed as numbers.
/// Chars are preceeded with a quote `'`.
fn params(input: &str) -> FormatResult<Params> {
    map(separated_list0(tag(","), param), Params::new)(input)
}

/// Directives can be modified with a colon (`:`) or an ampersat (`@`).
/// Modifiers have different meanings depending on the directive.
struct Modifiers {
    colon: bool,
    at: bool,
}

fn modifiers(input: &str) -> FormatResult<Modifiers> {
    let (input, modifiers) = take_while(|c| c == ':' || c == '@')(input)?;

    Ok((
        input,
        Modifiers {
            colon: modifiers.contains(':'),
            at: modifiers.contains('@'),
        },
    ))
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
                Directive::Decimal {
                    min_columns: 0,
                    pad_char: ' ',
                    comma_char: ',',
                    comma_interval: 3,
                    print_commas: false,
                    print_sign: false,
                },
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

    #[test]
    fn parses_alignment() {
        let format_string = "zork ~10<~A~>~%";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string).unwrap();

        assert_eq!(
            vec![
                Directive::Literal("zork ".to_string()),
                Directive::Align {
                    inner: vec![Directive::TildeA],
                    min_columns: 10,
                    col_inc: 0,
                    min_pad: 0,
                    pad_char: ' ',
                },
                Directive::Newline,
            ],
            parsed
        );
    }

    #[test]
    fn errors_on_invalid_directive() {
        let format_string = "Ook, ~z";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string);
        assert_eq!(
            Err("invalid directive `~Z`".to_string()),
            parsed.map_err(|err| err.to_string())
        );
    }

    #[test]
    fn errors_on_break_outside_loop() {
        let format_string = "Oook ~^ ~{~A}";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string);
        assert_eq!(
            Err("directive `^` not inside loop".to_string()),
            parsed.map_err(|err| err.to_string())
        );
    }
}
