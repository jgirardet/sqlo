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
mod sqlo_update;
mod types;
mod utils;
mod virtual_table;

use crate::parse::SqloParse;
use crate::sqlo::Sqlo;
use darling::FromDeriveInput;
// use macros::sqlo_select::{process_sqlo_select, SqloSelectParse};
use proc_macro2::TokenStream;
use sqlo_update::{process_sqlo_set, SqloSetParse};
use virtual_table::VirtualFile;

fn process_all(deriveinput: ::syn::DeriveInput) -> syn::Result<TokenStream> {
    let sqlo: Sqlo = SqloParse::from_derive_input(&deriveinput)?.try_into()?;
    let vf = VirtualFile::new(sqlo.clone());
    vf.update()?;
    vf.validate()?;
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

// #[proc_macro]
// pub fn sqlo_select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let pts: SqloSelectParse = syn::parse_macro_input!(input as SqloSelectParse);
//     match process_sqlo_select(pts) {
//         Ok(ts) => ts.into(),
//         Err(e) => e.to_compile_error().into(),
//     }
// }
