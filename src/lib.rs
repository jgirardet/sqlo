#[macro_use]
mod utils;
mod error;
mod field;
mod macros;
mod methods;
mod parse;
mod produce;
mod query_builder;
mod relations;
mod serdable;
mod sqlo;
mod sqlos;
mod types;
mod virtual_file;

use crate::parse::SqloParse;
use crate::sqlo::Sqlo;
use darling::FromDeriveInput;
use macros::select::{Select, SelectAs};
use macros::sqlo_select::{process_sqlo_select, SqloSelectParse};
use macros::sqlo_update::{process_sqlo_set, SqloSetParse};
use proc_macro2::TokenStream;
use virtual_file::VirtualFile;

fn process_all(deriveinput: ::syn::DeriveInput) -> syn::Result<TokenStream> {
    let sqlo: Sqlo = SqloParse::from_derive_input(&deriveinput)?.try_into()?;
    let vf = VirtualFile::new();
    vf.update(&sqlo)?;
    vf.validate(&sqlo)?;
    if sqlo.parse_only {
        return Ok(TokenStream::new());
    }
    Ok(produce::produce(&sqlo))
}

#[proc_macro_derive(Sqlo, attributes(sqlo))]
pub fn macro_derive_sqlo(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let deriveinput = syn::parse_macro_input!(input as syn::DeriveInput);

    match process_all(deriveinput) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn sqlo_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pts: SqloSetParse = syn::parse_macro_input!(input as SqloSetParse);
    match process_sqlo_set(pts) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn sqlo_select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pts: SqloSelectParse = syn::parse_macro_input!(input as SqloSelectParse);
    match process_sqlo_select(pts) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pts: Select = syn::parse_macro_input!(input as Select);
    match pts.expand() {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn select_as(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pts: SelectAs = syn::parse_macro_input!(input as SelectAs);
    match pts.expand() {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn sqlo(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let sqlo_m: crate::macros::common::Phrase =
        syn::parse_macro_input!(input as crate::macros::common::Phrase);
    let sqlos = VirtualFile::new().load().unwrap();

    match sqlo_m.expand(&sqlos) {
        Ok(r) => r.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn sqlo_as(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let sqloas = syn::parse_macro_input!(input as crate::macros::common::SqloAs);
    let sqlos = VirtualFile::new().load().unwrap();

    match sqloas.expand(&sqlos) {
        Ok(r) => r.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
