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
        let mut expressions = self.expressions.iter();

        quote! {
            use std::fmt::Write;
            let mut result = String::new();
        }
        .to_tokens(tokens);

        for directive in &self.formatstr {
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
                Directive::TildePercent => {
                    quote! { write!(result, "\n").unwrap(); }.to_tokens(tokens)
                }
                Directive::Literal(literal) => {
                    quote! { write!(result, #literal).unwrap(); }.to_tokens(tokens)
                }
                Directive::Iteration(_) => todo!(),
            }
        }

        quote! { result }.to_tokens(tokens);
    }
}

#[proc_macro]
pub fn clformat(item: TokenStream) -> TokenStream {
    let ast: FormatInput = parse_macro_input!(item);

    quote!({ #ast }).into()
}
