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

enum Output {
    Writer(Expr),
    String,
    Stdout,
}

pub(crate) struct FormatInput {
    formatstr: Vec<Directive>,
    output: Output,
    expressions: Punctuated<Expr, Comma>,
}

impl std::fmt::Debug for FormatInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.formatstr)
    }
}

impl Parse for FormatInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let output: Expr = input.parse()?;
        let output = match output {
            Expr::Path(path) if path.path.is_ident("nil") => Output::String,
            Expr::Path(path) if path.path.is_ident("t") => Output::Stdout,
            expr => Output::Writer(expr),
        };
        let _: Comma = input.parse().expect("parse comma");

        let formatlit: LitStr = input.parse()?;
        let s = formatlit.value().clone();
        let formatstr = parse_format_string(formatlit, &s)?;

        let _: Comma = input.parse().expect("parse comma");
        let expressions = Punctuated::<Expr, Comma>::parse_terminated(input)?;

        Ok(Self {
            formatstr,
            output,
            expressions,
        })
    }
}

impl ToTokens for FormatInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut expressions = self.expressions.iter();

        let var_name: Expr = parse_quote!(__formatcl_result);

        let mut expr_tokens = proc_macro2::TokenStream::new();
        write_expressions(
            &mut expressions,
            &self.formatstr,
            &mut expr_tokens,
            var_name.clone(),
        );

        let uses = match self.output {
            Output::String => {
                quote! {
                    use ::std::fmt::Write;
                    let mut #var_name = String::new();
                }
            }
            Output::Stdout => {
                quote! {
                    use ::std::io::Write;
                    let mut #var_name = ::std::io::stdout();
                }
            }
            Output::Writer(ref expr) => {
                quote! {
                    let mut #var_name = &mut #expr;
                }
            }
        };

        quote! {
            #uses
            let __formatcl_err: Result<(), _> = '__format_cl__loop: loop {
                #expr_tokens
                break '__format_cl__loop Ok(());
            };

            if __formatcl_err.is_err() {
                panic!("oh no");
            }

            #var_name
        }
        .to_tokens(tokens);
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
                quote! {
                     let r = write!(#writer, "{}", #expression);
                     if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::TildeS => {
                let expression = expressions.next().expect("enough parameters");
                quote! {
                    let r = write!(#writer, "{:?}", #expression);
                    if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::Newline => quote! {
               let r = write!(#writer, "\n");
               if r.is_err() {
                   break '__format_cl__loop r;
               }
            }
            .to_tokens(tokens),
            Directive::Literal(literal) => quote! {
               let r = write!(#writer, #literal);
               if r.is_err() {
                   break '__format_cl__loop r;
               }
            }
            .to_tokens(tokens),
            Directive::Skip => {
                let expression = expressions.next().expect("enough parameters");
                // Note we have to output the expression since loop expressions involve side effects.
                quote! {  let _ = #expression; }.to_tokens(tokens)
            }
            Directive::Iteration(directives) => {
                let expression = expressions.next().expect("enough parameters");
                let iter = syn::parse_str::<Expr>("__formatcl_iteration.next().unwrap()")
                    .expect("static string should be valid syntax");
                let mut nested = IndexedExpression {
                    count: 0,
                    expr: &iter,
                };
                let mut block = proc_macro2::TokenStream::new();
                write_expressions(&mut nested, directives, &mut block, writer.clone());

                quote! {
                    let mut __formatcl_iteration = #expression.into_iter().peekable();
                    loop {
                        if __formatcl_iteration.peek().is_none() {
                            break;
                        }
                        { #block }
                    }
                }
                .to_tokens(tokens);
            }
            Directive::Break => {
                quote! {
                    if __formatcl_iteration.peek().is_none() {
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
                        let r = write!(#writer, "{}", __formatcl_c);
                        if r.is_err() {
                            break '__format_cl__loop r;
                        }
                    }
                }
                .to_tokens(tokens)
            }

            Directive::Float {
                width,
                num_decimal_places,
                pad_char,
            } => {
                let expression = expressions.next().expect("enough parameters");
                let format = format!(
                    "{{:{pad_char}>{width}.{num_decimal_places}}}",
                    width = if *width == 0 {
                        String::new()
                    } else {
                        width.to_string()
                    },
                    num_decimal_places = if *num_decimal_places == 0 {
                        String::new()
                    } else {
                        num_decimal_places.to_string()
                    }
                );
                quote! {
                    let r = write!(#writer, #format, #expression);
                    if r.is_err() {
                        break '__format_cl__loop r;
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
                        if #min_columns > #ruler_var.length() {
                            let r = write!(#writer, #fill, "", width = #min_columns - #ruler_var.length());
                            if r.is_err() {
                                break '__format_cl__loop r;
                            }
                        }
                    },
                    Alignment::Centre => quote! {
                        if #min_columns > #ruler_var.length() {
                            let r = write!(#writer, #fill, "", width = (#min_columns - #ruler_var.length()) / 2);
                            if r.is_err() {
                                break '__format_cl__loop r;
                            }
                        }
                    },
                };

                let right_fill = match direction {
                    Alignment::Left => quote! {
                        if #min_columns > #ruler_var.length() {
                            let r = write!(#writer, #fill, "", width = #min_columns - #ruler_var.length());
                            if r.is_err() {
                                break '__format_cl__loop r;
                            }
                        }
                    },
                    Alignment::Right => Default::default(),
                    Alignment::Centre => quote! {
                        if #min_columns > #ruler_var.length() {
                            let r = write!(#writer, #fill, "", width = (#min_columns - #ruler_var.length()) / 2);
                            if r.is_err() {
                                break '__format_cl__loop r;
                            }
                        }
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
