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
use crate::parse::EnglishMode;
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

        // `#var_name` is always wrapped in a `ColumnTracker` so that
        // directives such as `~T` (tabulate) and `~&` (fresh-line) can know
        // the current output column regardless of the output destination.
        let uses = match self.output {
            Output::String => {
                quote! {
                    use ::std::fmt::Write;
                    let mut #var_name = ::clformat::ColumnTracker::new(String::new());
                }
            }
            Output::Stdout => {
                quote! {
                    use ::std::fmt::Write;
                    let mut #var_name = ::clformat::ColumnTracker::new(
                        ::clformat::IoWriteAdapter(::std::io::stdout()),
                    );
                }
            }
            Output::Writer(ref expr) => {
                quote! {
                    use ::std::fmt::Write;
                    let mut #var_name = ::clformat::ColumnTracker::new(&mut #expr);
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

            #var_name.into_inner()
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
    // The most recently consumed argument expression, tracked so that `~:P`
    // (and `~:@P`) can refer back to it without consuming (or
    // re-evaluating) a new argument.
    let mut last_expr: Option<&'a Expr> = None;

    for directive in directives {
        match directive {
            Directive::TildeA => {
                let expression = expressions.next().expect("enough parameters");
                last_expr = Some(expression);
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
                last_expr = Some(expression);
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
                last_expr = Some(expression);
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
            Directive::Conditional {
                boolean: true,
                consumes: _,
                choices,
                default: _,
            } => {
                let expression = expressions.next().expect("enough parameters");

                let then = {
                    let mut block = proc_macro2::TokenStream::new();
                    // We have already validated to ensure there are exactly two choices.
                    write_expressions(
                        &mut expressions.clone(),
                        &choices[0],
                        &mut block,
                        writer.clone(),
                    );
                    block
                };

                let r#else = {
                    let mut block = proc_macro2::TokenStream::new();
                    // We have already validated to ensure there are exactly two choices.
                    write_expressions(
                        &mut expressions.clone(),
                        &choices[1],
                        &mut block,
                        writer.clone(),
                    );
                    block
                };

                quote! {
                    if #expression {
                        #then
                    } else {
                        #r#else
                    }
                }
                .to_tokens(tokens);
            }
            Directive::Conditional {
                boolean: false,
                consumes,
                choices,
                default,
            } => {
                let expression = expressions.next().expect("enough parameters");

                if *consumes {
                    // We should have validated there is only one choice when `consumes` is true.
                    let choice = &choices[0];
                    let mut block = proc_macro2::TokenStream::new();
                    let expr = syn::parse_str::<Expr>("__formatcl_inner")
                        .expect("static string should be valid syntax");
                    let mut inner = IndexedExpression {
                        count: 0,
                        expr: &expr,
                    };

                    write_expressions(&mut inner, &choice, &mut block, writer.clone());

                    quote! {
                        if let ::core::option::Option::Some(__formatcl_inner) = #expression {
                            #block
                        }
                    }
                    .to_tokens(tokens);
                } else {
                    let mut match_tokens = quote! {};

                    for (idx, e) in choices.iter().enumerate() {
                        let mut block = proc_macro2::TokenStream::new();
                        write_expressions(&mut expressions.clone(), e, &mut block, writer.clone());
                        match_tokens = quote! {
                            #match_tokens
                            #idx => { #block }
                        };
                    }

                    if let Some(default) = default {
                        let mut block = proc_macro2::TokenStream::new();
                        write_expressions(
                            &mut expressions.clone(),
                            default,
                            &mut block,
                            writer.clone(),
                        );

                        match_tokens = quote! {
                            #match_tokens
                            _ => { #block }
                        }
                    }

                    quote! {
                        match #expression {
                            #match_tokens
                        }
                    }
                    .to_tokens(tokens);
                }
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
                radix,
                min_columns,
                pad_char,
                comma_char,
                comma_interval,
                print_commas,
                print_sign,
            } => {
                let expression = expressions.next().expect("enough parameters");
                last_expr = Some(expression);
                quote! {
                    for __formatcl_c in ::clformat::Decimal::new_radix(
                                             #radix as usize,
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
                last_expr = Some(expression);
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
            Directive::Exponential {
                width,
                digits,
                pad_char,
                exponent_char,
                print_sign,
            } => {
                let expression = expressions.next().expect("enough parameters");
                last_expr = Some(expression);
                let digits_tokens = match digits {
                    Some(d) => quote! { ::core::option::Option::Some(#d) },
                    None => quote! { ::core::option::Option::None },
                };
                let format = format!(
                    "{{:{pad_char}>{width}}}",
                    width = if *width == 0 {
                        String::new()
                    } else {
                        width.to_string()
                    }
                );

                quote! {
                    let __formatcl_value: f64 = (#expression) as f64;
                    let __formatcl_digits: ::core::option::Option<usize> = #digits_tokens;
                    let __formatcl_s = ::clformat::format_exponential(
                        __formatcl_value,
                        __formatcl_digits.unwrap_or(6),
                        #exponent_char,
                    );
                    let __formatcl_s = if #print_sign && __formatcl_value >= 0.0 {
                        ::std::format!("+{}", __formatcl_s)
                    } else {
                        __formatcl_s
                    };
                    let r = write!(#writer, #format, __formatcl_s);
                    if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::General {
                width,
                digits,
                pad_char,
                exponent_char,
                print_sign,
            } => {
                let expression = expressions.next().expect("enough parameters");
                last_expr = Some(expression);
                let digits_tokens = match digits {
                    Some(d) => quote! { ::core::option::Option::Some(#d) },
                    None => quote! { ::core::option::Option::None },
                };
                let format = format!(
                    "{{:{pad_char}>{width}}}",
                    width = if *width == 0 {
                        String::new()
                    } else {
                        width.to_string()
                    }
                );

                quote! {
                    let __formatcl_value: f64 = (#expression) as f64;
                    let __formatcl_digits: ::core::option::Option<usize> = #digits_tokens;
                    let (__formatcl_exp, __formatcl_places) =
                        ::clformat::general_params(__formatcl_value, __formatcl_digits);
                    let __formatcl_s = if __formatcl_exp {
                        ::clformat::format_exponential(__formatcl_value, __formatcl_places, #exponent_char)
                    } else {
                        ::std::format!("{:.*}", __formatcl_places, __formatcl_value)
                    };
                    let __formatcl_s = if #print_sign && __formatcl_value >= 0.0 {
                        ::std::format!("+{}", __formatcl_s)
                    } else {
                        __formatcl_s
                    };
                    let r = write!(#writer, #format, __formatcl_s);
                    if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::EnglishNumber(mode) => {
                let expression = expressions.next().expect("enough parameters");
                last_expr = Some(expression);
                let call = match mode {
                    EnglishMode::Cardinal => {
                        quote! { ::clformat::to_cardinal((#expression) as i128) }
                    }
                    EnglishMode::Ordinal => {
                        quote! { ::clformat::to_ordinal((#expression) as i128) }
                    }
                    EnglishMode::Roman => {
                        quote! { ::clformat::to_roman((#expression) as i128, false) }
                    }
                    EnglishMode::OldRoman => {
                        quote! { ::clformat::to_roman((#expression) as i128, true) }
                    }
                };
                quote! {
                    let r = write!(#writer, "{}", #call);
                    if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::Char { named, lisp_syntax } => {
                let expression = expressions.next().expect("enough parameters");
                last_expr = Some(expression);

                let body = match (named, lisp_syntax) {
                    (false, false) => quote! {
                        write!(#writer, "{}", #expression)
                    },
                    (true, false) => quote! {
                        match ::clformat::char_name(#expression) {
                            ::core::option::Option::Some(name) => write!(#writer, "{}", name),
                            ::core::option::Option::None => write!(#writer, "{}", #expression),
                        }
                    },
                    (_, true) => quote! {
                        match ::clformat::char_name(#expression) {
                            ::core::option::Option::Some(name) => write!(#writer, "#\\{}", name),
                            ::core::option::Option::None => write!(#writer, "#\\{}", #expression),
                        }
                    },
                };

                quote! {
                    let r = #body;
                    if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::Plural {
                y_form,
                use_previous,
            } => {
                let expression = if *use_previous {
                    last_expr
                        .expect("~:P must be preceded by a directive that consumes an argument")
                } else {
                    let expression = expressions.next().expect("enough parameters");
                    last_expr = Some(expression);
                    expression
                };

                let text = if *y_form {
                    quote! { if (#expression) == 1 { "y" } else { "ies" } }
                } else {
                    quote! { if (#expression) == 1 { "" } else { "s" } }
                };

                quote! {
                    let r = write!(#writer, "{}", #text);
                    if r.is_err() {
                        break '__format_cl__loop r;
                    }
                }
                .to_tokens(tokens)
            }
            Directive::Tilde => quote! {
               let r = write!(#writer, "~");
               if r.is_err() {
                   break '__format_cl__loop r;
               }
            }
            .to_tokens(tokens),
            Directive::FreshLine => quote! {
               let r = if #writer.column() != 0 {
                   write!(#writer, "\n")
               } else {
                   Ok(())
               };
               if r.is_err() {
                   break '__format_cl__loop r;
               }
            }
            .to_tokens(tokens),
            Directive::Tab {
                colnum,
                colinc,
                relative,
            } => {
                let target = if *relative {
                    quote! {
                        {
                            let __formatcl_target = #writer.column() + #colnum;
                            if #colinc > 0 {
                                __formatcl_target + ((#colinc - (__formatcl_target % #colinc)) % #colinc)
                            } else {
                                __formatcl_target
                            }
                        }
                    }
                } else {
                    quote! {
                        {
                            let __formatcl_current = #writer.column();
                            if __formatcl_current <= #colnum {
                                #colnum
                            } else if #colinc > 0 {
                                let __formatcl_diff = __formatcl_current - #colnum;
                                #colnum + __formatcl_diff.div_ceil(#colinc) * #colinc
                            } else {
                                __formatcl_current
                            }
                        }
                    }
                };

                quote! {
                    let __formatcl_tab_target = #target;
                    let __formatcl_tab_pad = __formatcl_tab_target.saturating_sub(#writer.column());
                    let r = write!(#writer, "{:1$}", "", __formatcl_tab_pad);
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
