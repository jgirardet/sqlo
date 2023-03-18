use darling::util::IdentString;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Token,
};

use crate::macros::Column;

use super::next_is_not_a_keyword;

pub fn parse_identstring(input: ParseStream) -> syn::Result<IdentString> {
    input.parse::<syn::Ident>().map(|x| x.into())
}

pub fn parse_sqlo_struct_ident(input: ParseStream) -> syn::Result<IdentString> {
    input
        .call(parse_identstring)
        .map_err(|_| input.error("Derived Sqlo struct expected"))
}

pub fn parse_optional_ident_with_comma(input: ParseStream) -> syn::Result<Option<IdentString>> {
    if input.peek2(Token![,]) {
        let ident = input.call(parse_identstring)?;
        input.parse::<Token![,]>()?;
        Ok(Some(ident))
    } else {
        Ok(None)
    }
}

pub fn parse_bracketed(input: ParseStream) -> syn::Result<Expr> {
    let content;
    syn::bracketed!(content in input);
    content.parse::<syn::Expr>()
}

pub fn parse_optional_bracketed(input: ParseStream) -> syn::Result<Option<Expr>> {
    if input.peek(syn::token::Bracket) {
        input.call(parse_bracketed).map(|x| x.into())
    } else {
        Ok(None)
    }
}

pub fn parse_field_member(input: ParseStream) -> syn::Result<IdentString> {
    input.parse::<Token![.]>()?;
    input.call(parse_identstring)
}

pub fn parse_optional_field_member(input: ParseStream) -> syn::Result<Option<IdentString>> {
    if input.peek(Token![.]) {
        input.parse::<Token![.]>()?;
        input.call(parse_identstring).map(|x| x.into())
    } else {
        Ok(None)
    }
}

pub fn parse_columns(input: ParseStream) -> syn::Result<Punctuated<Column, Token![,]>> {
    Punctuated::parse_separated_nonempty(input)
}

pub fn parse_optional_columns(
    input: ParseStream,
) -> syn::Result<Option<Punctuated<Column, Token![,]>>> {
    if !input.is_empty() && next_is_not_a_keyword(&input) {
        input.call(parse_columns).map(|x| x.into())
    } else {
        Ok(None)
    }
}

macro_rules! impl_parse_optional_clauses {
    (# $($($tok:ident),+ $target_struct:ident);+) => {
    paste::paste! {
    $(
    pub fn [<parse_optional_ $($tok)_+>](input: syn::parse::ParseStream) -> syn::Result<Option<$crate::macros::Clause>> {
        if !input.is_empty() && $(input.peek(syn::Token![$tok]))||+ {
            input.parse::<$crate::macros::$target_struct>().map(|x| Some(x.into()))
        } else {
            Ok(None)
        }
    }
        )+
    }
    };
    ($($($tok:ident),+ $target_struct:ident);+) => {
    paste::paste! {
    $(
    pub fn [<parse_optional_ $($tok)_+>](input: syn::parse::ParseStream) -> syn::Result<Option<$crate::macros::Clause>> {
        if !input.is_empty() && ($(input.peek($crate::macros::kw::$tok))||+) {
            input.parse::<$crate::macros::$target_struct>().map(|x| Some(x.into()))
        } else {
            Ok(None)
        }
    }
        )+
    }
    };
}

impl_parse_optional_clauses! {
group_by GroupBy;
having Having;
order_by OrderBy;
limit,page Limit
}

impl_parse_optional_clauses! {# where Where}

#[cfg(debug_assertions)]
pub fn parse_dbg_symbol(input: ParseStream) -> syn::Result<bool> {
    let fork = input.fork();
    let ident = syn::Ident::parse(&fork).unwrap_or_else(|_| syn::Ident::new(".", input.span()));
    if &ident.to_string() == "dbg" && fork.peek(Token![!]) {
        syn::Ident::parse(input).unwrap();
        input.parse::<Token!(!)>().unwrap();
        Ok(true)
    } else {
        Ok(false)
    }
}
