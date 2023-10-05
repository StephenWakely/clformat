use std::iter::Peekable;

use syn::{token::Token, LitStr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Directive {
    TildeA,
    TildeS,
    TildeD,
    TildePercent,
    Iteration(Vec<Directive>),
    Literal(String),
}

pub fn parse_format_string(
    token: LitStr,
    format_string: &str,
) -> Result<Vec<Directive>, syn::Error> {
    let mut chars = format_string.chars().peekable();
    parse_string(&mut chars, token)
}

fn parse_string(
    chars: &mut Peekable<std::str::Chars>,
    token: LitStr,
) -> Result<Vec<Directive>, syn::Error> {
    let mut directives = Vec::new();
    let mut literal = String::new();
    while let Some(c) = chars.next() {
        if c == '~' {
            if !literal.is_empty() {
                directives.push(Directive::Literal(literal));
                literal = String::new();
            }

            match chars.peek() {
                Some('A') => {
                    directives.push(Directive::TildeA);
                    chars.next();
                }
                Some('S') => {
                    directives.push(Directive::TildeS);
                    chars.next();
                }
                Some('D') => {
                    directives.push(Directive::TildeD);
                    chars.next();
                }
                Some('%') => {
                    directives.push(Directive::TildePercent);
                    chars.next();
                }
                Some('{') => {
                    chars.next();
                    let iteration = parse_string(chars, token.clone())?;
                    directives.push(Directive::Iteration(iteration));
                }
                Some('}') => {
                    chars.next();
                    return Ok(directives);
                }
                Some(directive) => {
                    return Err(syn::Error::new_spanned(
                        token,
                        format!("Invalid directive: {directive}"),
                    ))
                }
                None => {
                    // Lone tilde at the end of the string
                }
            }
        } else {
            literal.push(c);
        }
    }

    Ok(directives)
}

#[cfg(test)]
mod tests {
    use super::*;

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
                Directive::TildePercent
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
                Directive::TildePercent
            ],
            parsed
        );
    }
}
