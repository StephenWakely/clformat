use quote::quote;
use quote::ToTokens;
use syn::parse_quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Expr, LitStr,
};

use crate::parse::Alignment;
use crate::parse::{parse_format_string, Directive};

pub(crate) struct FormatInput {
    formatstr: Vec<Directive>,
    expressions: Punctuated<Expr, Comma>,
}

impl std::fmt::Debug for FormatInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.formatstr)
    }
}

impl Parse for FormatInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let formatlit: LitStr = input.parse()?;
        let s = formatlit.value().clone();
        let formatstr = parse_format_string(formatlit, &s)?;

        let _: Comma = input.parse().expect("parse comma");
        let expressions = Punctuated::<Expr, Comma>::parse_terminated(input)?;

        Ok(Self {
            formatstr,
            expressions,
        })
    }
}

impl ToTokens for FormatInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut expressions = self.expressions.iter();

        let var_name: Expr = parse_quote!(__formatcl_result);

        quote! {
            use std::fmt::Write;
            let mut #var_name = String::new();
        }
        .to_tokens(tokens);

        write_expressions(&mut expressions, &self.formatstr, tokens, var_name.clone());

        var_name.to_tokens(tokens);
    }
}

fn write_expressions<'a, T>(
    expressions: &mut T,
    directives: &[Directive],
    tokens: &mut proc_macro2::TokenStream,
    writer: Expr,
) where
    T: Iterator<Item = &'a Expr> + Clone,
{
    for directive in directives {
        match directive {
            Directive::TildeA => {
                let expression = expressions.next().expect("enough parameters");
                quote! { write!(#writer, "{}", #expression).unwrap(); }.to_tokens(tokens)
            }
            Directive::TildeS => {
                let expression = expressions.next().expect("enough parameters");
                quote! { write!(#writer, "{:?}", #expression).unwrap(); }.to_tokens(tokens)
            }
            Directive::Newline => quote! { write!(#writer, "\n").unwrap(); }.to_tokens(tokens),
            Directive::Literal(literal) => {
                quote! { write!(#writer, #literal).unwrap(); }.to_tokens(tokens)
            }
            Directive::Skip => {
                let expression = expressions.next().expect("enough parameters");
                // Note we have to output the expression since loop expressions involve side effects.
                quote! {  let _ = #expression; }.to_tokens(tokens)
            }
            Directive::Iteration(directives) => {
                let expression = expressions.next().expect("enough parameters");
                let iter = syn::parse_str::<Expr>("zork.next().unwrap()")
                    .expect("static string should be valid syntax");
                let mut nested = IndexedExpression {
                    count: 0,
                    expr: &iter,
                };
                let mut block = proc_macro2::TokenStream::new();
                write_expressions(&mut nested, directives, &mut block, writer.clone());

                quote! {
                    let mut zork = #expression.into_iter().peekable();
                    loop {
                        if zork.peek().is_none() {
                            break;
                        }
                        { #block }
                    }
                }
                .to_tokens(tokens);
            }
            Directive::Break => {
                quote! {
                    if zork.peek().is_none() {
                        break;
                    }
                }
                .to_tokens(tokens);
            }
            Directive::Decimal {
                min_columns,
                pad_char,
                comma_char,
                comma_interval,
                print_commas,
                print_sign,
            } => {
                let expression = expressions.next().expect("enough parameters");
                quote! {
                    for __formatcl_c in ::clformat::Decimal::new(
                                             #min_columns,
                                             #pad_char,
                                             #comma_char,
                                             #comma_interval,
                                             #print_commas,
                                             #print_sign,
                                             #expression) {
                        write!(#writer, "{}", __formatcl_c).unwrap();
                    }
                }
                .to_tokens(tokens)
            }
            Directive::Align {
                min_columns,
                pad_char,
                direction,
                inner,
                ..
            } => {
                // Alignment is achieved by writing the contained tokens twice.
                // First we write to a ruler writer which measures the length of the text, this is used to
                // calculate the padding, which we then output before writing the blocks properly.
                let mut ruler_block = proc_macro2::TokenStream::new();
                let mut writer_block = proc_macro2::TokenStream::new();
                let ruler_var: Expr = parse_quote!(__formatcl_ruler);
                write_expressions(
                    &mut expressions.clone(),
                    inner,
                    &mut ruler_block,
                    ruler_var.clone(),
                );
                write_expressions(expressions, inner, &mut writer_block, writer.clone());

                let fill = format!("{{:{pad_char}<width$}}");

                let left_fill = match direction {
                    Alignment::Left => Default::default(),
                    Alignment::Right => quote! {
                        write!(#writer, #fill, "", width = #min_columns - #ruler_var.length()).unwrap();
                    },
                    Alignment::Centre => quote! {
                        write!(#writer, #fill, "", width = (#min_columns - #ruler_var.length()) / 2).unwrap();
                    },
                };

                let right_fill = match direction {
                    Alignment::Left => quote! {
                        write!(#writer, #fill, "", width = #min_columns - #ruler_var.length()).unwrap();
                    },
                    Alignment::Right => Default::default(),
                    Alignment::Centre => quote! {
                        write!(#writer, #fill, "", width = (#min_columns - #ruler_var.length()) / 2).unwrap();
                    },
                };

                quote! {
                    let mut #ruler_var = ::clformat::Ruler::default();
                    #ruler_block

                    #left_fill
                    #writer_block
                    #right_fill
                }
                .to_tokens(tokens)
            }
        }
    }
}

#[derive(Clone)]
struct IndexedExpression<'a> {
    count: usize,
    expr: &'a Expr,
}

impl<'a> Iterator for IndexedExpression<'a> {
    type Item = &'a Expr;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        Some(self.expr)
    }
}
