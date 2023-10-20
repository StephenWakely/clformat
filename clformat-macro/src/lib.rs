use parse::{parse_format_string, Directive};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    Expr, LitStr,
};

mod parse;

struct FormatInput {
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

        let _: Comma = input.parse().unwrap();
        let expressions = Punctuated::<Expr, Comma>::parse_terminated(input)?;

        Ok(Self {
            formatstr,
            expressions,
        })
    }
}

impl ToTokens for FormatInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expressions = self.expressions.iter();

        quote! {
            use std::fmt::Write;
            let mut result = String::new();
        }
        .to_tokens(tokens);

        self.write_expressions(expressions, &self.formatstr, tokens);

        quote! { result }.to_tokens(tokens);
    }
}

impl FormatInput {
    fn write_expressions<'a, T>(
        &self,
        mut expressions: T,
        directives: &[Directive],
        tokens: &mut proc_macro2::TokenStream,
    ) where
        T: Iterator<Item = &'a Expr>,
    {
        for directive in directives {
            match directive {
                Directive::TildeA => {
                    let expression = expressions.next().unwrap();
                    quote! { write!(result, "{}", #expression).unwrap(); }.to_tokens(tokens)
                }
                Directive::TildeS => {
                    let expression = expressions.next().unwrap();
                    quote! { write!(result, "{:?}", #expression).unwrap(); }.to_tokens(tokens)
                }
                Directive::TildeD => {
                    let expression = expressions.next().unwrap();
                    quote! { write!(result, "{}", #expression).unwrap(); }.to_tokens(tokens)
                }
                Directive::Newline => quote! { write!(result, "\n").unwrap(); }.to_tokens(tokens),
                Directive::Literal(literal) => {
                    quote! { write!(result, #literal).unwrap(); }.to_tokens(tokens)
                }
                Directive::Skip => {
                    let expression = expressions.next().unwrap();
                    // Note we have to output the expression since loop expressions involve side effects.
                    quote! {  let _ = #expression; }.to_tokens(tokens)
                }
                Directive::Iteration(directives) => {
                    let expression = expressions.next().unwrap();
                    let iter = syn::parse_str::<Expr>("zork.next().unwrap()")
                        .expect("static string should be valid syntax");
                    let nested = IndexedExpression {
                        count: 0,
                        expr: &iter,
                    };
                    let mut block = proc_macro2::TokenStream::new();
                    self.write_expressions(nested, directives, &mut block);

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
                #[allow(warnings)]
                Directive::Decimal {
                    min_columns,
                    pad_char,
                    comma_char,
                    comma_interval,
                } => todo!(),
            }
        }
    }
}

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

#[proc_macro]
pub fn clformat(item: TokenStream) -> TokenStream {
    let ast: FormatInput = parse_macro_input!(item);

    quote!({ #ast }).into()
}
