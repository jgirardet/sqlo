use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::ToTokens;

pub fn compile_error<T: ToTokens, U: Display>(tokens: T, message: U) -> TokenStream {
    syn::Error::new_spanned(tokens, message).to_compile_error()
}

macro_rules! parse_possible_bracketed {
    ($input:expr, $reste:ident) => {
        let content;
        $reste = if $input.peek(syn::token::Bracket) {
        syn::bracketed!(content in $input);
        &content
        } else {
        $input
         }
    };
}
