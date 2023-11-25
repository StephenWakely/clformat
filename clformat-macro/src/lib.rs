use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod format_input;
mod parse;
mod parse_error;

use format_input::FormatInput;

#[proc_macro]
pub fn clformat(item: TokenStream) -> TokenStream {
    let ast: FormatInput = parse_macro_input!(item);

    quote!({ #ast }).into()
}
