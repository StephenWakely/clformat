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
pub enum Alignment {
    Left,
    Right,
    Centre,
}

impl From<Modifiers> for Alignment {
    fn from(value: Modifiers) -> Self {
        match (value.colon, value.at) {
            (true, true) => Self::Centre,
            (true, false) => Self::Right,
            (false, true) => Self::Left,
            (false, false) => Self::Left,
        }
    }
}

/// The Directives that are supported.
/// Attempts to conform to the [Hyperspec].
///
/// [Hyperspec]: https://www.lispworks.com/documentation/HyperSpec/Body/22_c.htm
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Directive {
    Align {
        min_columns: usize,
        col_inc: usize,
        min_pad: usize,
        pad_char: char,
        direction: Alignment,
        inner: Vec<Directive>,
    },
    Break,
    Conditional {
        boolean: bool,
        consumes: bool,
        default: Option<Vec<Directive>>,
        choices: Vec<Vec<Directive>>,
    },
    Decimal {
        radix: u32,
        min_columns: usize,
        pad_char: char,
        comma_char: char,
        comma_interval: usize,
        print_commas: bool,
        print_sign: bool,
    },
    Float {
        width: usize,
        num_decimal_places: usize,
        pad_char: char,
    },
    Exponential {
        width: usize,
        digits: Option<usize>,
        pad_char: char,
        exponent_char: char,
        print_sign: bool,
    },
    General {
        width: usize,
        digits: Option<usize>,
        pad_char: char,
        exponent_char: char,
        print_sign: bool,
    },
    EnglishNumber(EnglishMode),
    Char {
        named: bool,
        lisp_syntax: bool,
    },
    Plural {
        y_form: bool,
        use_previous: bool,
    },
    /// `~&` (and, approximately, `~_`): moves to the start of a new line
    /// unless already there.
    FreshLine,
    Tab {
        colnum: usize,
        colinc: usize,
        relative: bool,
    },
    Iteration(Vec<Directive>),
    Literal(String),
    Newline,
    Skip,
    Tilde,
    TildeA,
    TildeS,
}

/// The behaviour of `~R` when no explicit radix parameter is given (and of
/// `~M`, which is always one of the Roman-numeral modes).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EnglishMode {
    /// `~R` - e.g. "forty-two".
    Cardinal,
    /// `~:R` - e.g. "forty-second".
    Ordinal,
    /// `~@R` - e.g. "XLII".
    Roman,
    /// `~:@R` - old-style Roman numerals that never use subtractive
    /// notation, e.g. "VIIII" rather than "IX".
    OldRoman,
}

impl Directive {
    fn new_conditional<'a>(
        input: &'a str,
        boolean: bool,
        consumes: bool,
        choices: Vec<Vec<Directive>>,
        default: Option<Vec<Directive>>,
    ) -> Result<Self, nom::Err<FormatError<&'a str>>> {
        if boolean && (choices.len() != 2 || default.is_some()) {
            return Err(nom::Err::Error(FormatError::from_external_error(
                input,
                nom::error::ErrorKind::Tag,
                "boolean conditional must specify exactly two sections",
            )));
        }

        if consumes && (choices.len() != 1 || default.is_some()) {
            return Err(nom::Err::Error(FormatError::from_external_error(
                input,
                nom::error::ErrorKind::Tag,
                "consume conditional must specify exactly one section",
            )));
        }

        Ok(Self::Conditional {
            boolean,
            consumes,
            choices,
            default,
        })
    }
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
    move |input| alt((literal, alignment, iteration, conditional, directive(state)))(input)
}

fn literal(input: &str) -> FormatResult<Directive> {
    map(take_till1(|c| c == '~'), |s: &str| {
        Directive::Literal(s.to_string())
    })(input)
}

fn params_to_align(
    params: Params,
    modifiers: Modifiers,
    inner: Vec<Directive>,
) -> Result<Directive, String> {
    let min_columns = params.get_num(0, 0)? as usize;
    let col_inc = params.get_num(1, 0)? as usize;
    let min_pad = params.get_num(2, 0)? as usize;
    let pad_char = params.get_char(3, ' ')?;

    Ok((Directive::Align {
        min_columns,
        col_inc,
        min_pad,
        pad_char,
        direction: modifiers.into(),
        inner,
    }))
}

/// Conditional is a series of directive separated by `~:` and
/// enclosed by `~[..~]`.
fn conditional(input: &str) -> FormatResult<Directive> {
    let (input, _) = tag("~")(input)?;
    let (input, params) = params(input)?;
    let (input, modifiers) = modifiers(input)?;
    let (mut input, _) = tag("[")(input)?;

    let mut choices = Vec::new();
    let mut current = Vec::new();
    let mut default = None;
    let boolean = modifiers.colon;
    let consumes = modifiers.at;

    loop {
        if input.starts_with("~]") {
            if !current.is_empty() {
                choices.push(current);
            }

            return Ok((
                &input[2..],
                Directive::new_conditional(input, boolean, consumes, choices, default)?,
            ));
        } else if input.is_empty() {
            // Be permissive.
            return Ok((
                &input,
                Directive::new_conditional(input, boolean, consumes, choices, default)?,
            ));
        } else if input.starts_with("~;") {
            if default.is_some() {
                return Err(nom::Err::Error(FormatError::from_external_error(
                    input,
                    nom::error::ErrorKind::Tag,
                    "only the last conditional can be default",
                )));
            }

            // We are at the start of a new choice
            choices.push(std::mem::take(&mut current));
            input = &input[2..];
        } else if input.starts_with("~:;") {
            // The default case.
            choices.push(std::mem::take(&mut current));
            default = Some(vec![]);
            input = &input[3..];
        } else {
            let (new_input, directive) = segment(State::Loop)(input)?;
            input = new_input;

            match &mut default {
                Some(default) => default.push(directive),
                None => current.push(directive),
            }
        }
    }
}

/// Alignment is a series of directives enclosed by `~<..~>`.
/// There can optionally be params and modifiers to determine how to align
/// the enclosed directives.
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
                params_to_align(params, modifiers, result).map_err(|err| {
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
                params_to_align(params, modifiers, result).map_err(|err| {
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

/// Iteration as a series of directives enclosed by `~{..~}`.
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

/// Parse the directive - a supported character preceeded by a `~`.
fn directive(state: State) -> impl Fn(&str) -> FormatResult<Directive> {
    move |input| {
        map_res(
            preceded(tag("~"), tuple((params, modifiers, anychar))),
            |(params, modifiers, directive)| match directive.to_ascii_uppercase() {
                'A' => Ok(Directive::TildeA),
                'S' => Ok(Directive::TildeS),
                'W' => Ok(Directive::TildeS),
                'D' => decimal_directive(&params, &modifiers, 10),
                'X' => decimal_directive(&params, &modifiers, 16),
                'O' => decimal_directive(&params, &modifiers, 8),
                'B' => decimal_directive(&params, &modifiers, 2),
                'R' => match params.get_opt_num(0)? {
                    Some(radix) => {
                        if !(2..=36).contains(&radix) {
                            return Err(format!("radix must be between 2 and 36, got {radix}"));
                        }

                        let min_columns = params.get_num(1, 0)? as usize;
                        let pad_char = params.get_char(2, ' ')?;
                        let comma_char = params.get_char(3, ',')?;
                        let comma_interval = params.get_num(4, 3)? as usize;

                        Ok(Directive::Decimal {
                            radix: radix as u32,
                            min_columns,
                            pad_char,
                            comma_char,
                            comma_interval,
                            print_commas: modifiers.colon,
                            print_sign: modifiers.at,
                        })
                    }
                    None => Ok(Directive::EnglishNumber(
                        match (modifiers.colon, modifiers.at) {
                            (false, false) => EnglishMode::Cardinal,
                            (true, false) => EnglishMode::Ordinal,
                            (false, true) => EnglishMode::Roman,
                            (true, true) => EnglishMode::OldRoman,
                        },
                    )),
                },
                'M' => Ok(Directive::EnglishNumber(if modifiers.colon {
                    EnglishMode::OldRoman
                } else {
                    EnglishMode::Roman
                })),
                'C' => Ok(Directive::Char {
                    named: modifiers.colon,
                    lisp_syntax: modifiers.at,
                }),
                'P' => Ok(Directive::Plural {
                    y_form: modifiers.at,
                    use_previous: modifiers.colon,
                }),
                'F' => {
                    let width = params.get_num(0, 0)? as usize;
                    let num_decimal_places = params.get_num(1, 0)? as usize;
                    params.assert_missing(2, "num digits parameter not supported for floats")?;
                    params.assert_missing(3, "scale factor parameter not supported for floats")?;
                    params.assert_missing(4, "overflow char parameter not supported for floats")?;
                    let pad_char = params.get_char(5, ' ')?;

                    Ok(Directive::Float {
                        width,
                        num_decimal_places,
                        pad_char,
                    })
                }
                'E' => {
                    let width = params.get_num(0, 0)? as usize;
                    let digits = params.get_opt_num(1)?.map(|n| n as usize);
                    params.assert_missing(2, "exponent digits parameter not supported")?;
                    params.assert_missing(3, "scale factor parameter not supported")?;
                    params.assert_missing(4, "overflow char parameter not supported")?;
                    let pad_char = params.get_char(5, ' ')?;
                    let exponent_char = params.get_char(6, 'e')?;

                    Ok(Directive::Exponential {
                        width,
                        digits,
                        pad_char,
                        exponent_char,
                        print_sign: modifiers.at,
                    })
                }
                'G' => {
                    let width = params.get_num(0, 0)? as usize;
                    let digits = params.get_opt_num(1)?.map(|n| n as usize);
                    params.assert_missing(2, "exponent digits parameter not supported")?;
                    params.assert_missing(3, "scale factor parameter not supported")?;
                    params.assert_missing(4, "overflow char parameter not supported")?;
                    let pad_char = params.get_char(5, ' ')?;
                    let exponent_char = params.get_char(6, 'e')?;

                    Ok(Directive::General {
                        width,
                        digits,
                        pad_char,
                        exponent_char,
                        print_sign: modifiers.at,
                    })
                }
                'T' => {
                    let colnum = params.get_num(0, 1)? as usize;
                    let colinc = params.get_num(1, 1)? as usize;

                    Ok(Directive::Tab {
                        colnum,
                        colinc,
                        relative: modifiers.at,
                    })
                }
                '%' | 'N' => Ok(Directive::Newline),
                '&' | '_' => Ok(Directive::FreshLine),
                '~' => Ok(Directive::Tilde),
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

/// Shared parsing logic for `~D`, `~X`, `~O` and `~B`, which all take the
/// same parameters (minimum column count, pad character, comma character
/// and comma interval) and only differ in their (fixed) radix.
fn decimal_directive(
    params: &Params,
    modifiers: &Modifiers,
    radix: u32,
) -> Result<Directive, String> {
    let min_columns = params.get_num(0, 0)? as usize;
    let pad_char = params.get_char(1, ' ')?;
    let comma_char = params.get_char(2, ',')?;
    let comma_interval = params.get_num(3, 3)? as usize;

    Ok(Directive::Decimal {
        radix,
        min_columns,
        pad_char,
        comma_char,
        comma_interval,
        print_commas: modifiers.colon,
        print_sign: modifiers.at,
    })
}

#[derive(Debug, PartialEq, Eq)]
enum Param {
    Char(char),
    Num(isize),
    Missing,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Params {
    parsed: Vec<Param>,
}

impl Params {
    fn new(parsed: Vec<Param>) -> Self {
        Self { parsed }
    }

    /// For parameters we don't support yet.
    pub fn assert_missing(&self, idx: usize, msg: &str) -> Result<(), String> {
        match self.parsed.get(idx) {
            None | Some(Param::Missing) => Ok(()),
            _ => Err(msg.to_string()),
        }
    }

    pub fn get_num(&self, idx: usize, def: isize) -> Result<isize, String> {
        match self.parsed.get(idx) {
            Some(Param::Char(c)) => Err(format!("expected number, found char {c}")),
            Some(Param::Num(i)) => Ok(*i),
            Some(Param::Missing) => Ok(def),
            None => Ok(def),
        }
    }

    pub fn get_char(&self, idx: usize, def: char) -> Result<char, String> {
        match self.parsed.get(idx) {
            Some(Param::Num(i)) => Err(format!("expected character, found number {i}")),
            Some(Param::Char(c)) => Ok(*c),
            Some(Param::Missing) => Ok(def),
            None => Ok(def),
        }
    }

    /// Like [`Params::get_num`], but returns `None` (rather than a default
    /// value) when the parameter was not supplied at all. Used by
    /// directives such as `~R` and `~E` where the presence or absence of a
    /// parameter changes the directive's behaviour, not just its default.
    pub fn get_opt_num(&self, idx: usize) -> Result<Option<isize>, String> {
        match self.parsed.get(idx) {
            Some(Param::Char(c)) => Err(format!("expected number, found char {c}")),
            Some(Param::Num(i)) => Ok(Some(*i)),
            Some(Param::Missing) | None => Ok(None),
        }
    }
}

/// Parses a single parameter either:
/// -  an integer
/// -  or a single character preceeded by a quote (')
fn param(input: &str) -> FormatResult<Param> {
    alt((
        map(preceded(tag("'"), anychar), Param::Char),
        map(digit1, |nums: &str| {
            Param::Num(nums.parse().expect("numbers should have been parsed"))
        }),
        map(tag(""), |_: &str| Param::Missing),
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
                    radix: 10,
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
                    direction: Alignment::Left,
                },
                Directive::Newline,
            ],
            parsed
        );
    }

    #[test]
    fn parses_right_alignment() {
        let format_string = "zork ~10:<~A~>~%";
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
                    direction: Alignment::Right,
                },
                Directive::Newline,
            ],
            parsed
        );
    }

    #[test]
    fn parses_centre_alignment() {
        let format_string = "zork ~10:@<~A~>~%";
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
                    direction: Alignment::Centre,
                },
                Directive::Newline,
            ],
            parsed
        );
    }

    #[test]
    fn parse_conditional() {
        let format_string = "~[zork~;zoggle~;zoog~]";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string).unwrap();
        assert_eq!(
            vec![Directive::Conditional {
                boolean: false,
                consumes: false,
                default: None,
                choices: vec![
                    vec![Directive::Literal("zork".to_string())],
                    vec![Directive::Literal("zoggle".to_string())],
                    vec![Directive::Literal("zoog".to_string())],
                ]
            }],
            parsed
        );
    }

    #[test]
    fn parse_conditional_with_default() {
        let format_string = "~[zork~;zoggle~:;zoog~]";
        let token = LitStr::new("zork", proc_macro2::Span::call_site());
        let parsed = parse_format_string(token, format_string).unwrap();
        assert_eq!(
            vec![Directive::Conditional {
                boolean: false,
                consumes: false,
                choices: vec![
                    vec![Directive::Literal("zork".to_string())],
                    vec![Directive::Literal("zoggle".to_string())],
                ],
                default: Some(vec![Directive::Literal("zoog".to_string())]),
            }],
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

    #[test]
    fn parse_params() {
        let (_, res) = params("3,2,3").unwrap();
        assert_eq!(
            vec![Param::Num(3), Param::Num(2), Param::Num(3)],
            res.parsed
        );
    }

    #[test]
    fn parse_missing_params() {
        let (_, res) = params("3,,3").unwrap();
        assert_eq!(
            vec![Param::Num(3), Param::Missing, Param::Num(3)],
            res.parsed
        );

        let (_, res) = params(",9,3").unwrap();
        assert_eq!(
            vec![Param::Missing, Param::Num(9), Param::Num(3)],
            res.parsed
        );

        let (_, res) = params(",,3").unwrap();
        assert_eq!(
            vec![Param::Missing, Param::Missing, Param::Num(3)],
            res.parsed
        );
    }
}
