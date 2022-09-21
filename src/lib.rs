mod field;
mod methods;
mod parse;
mod produce;
mod query_builder;
mod serdable;
mod sqlo;
mod sqlo_set;
mod utils;
use crate::parse::SqloParse;
use crate::sqlo::Sqlo;
use crate::sqlo_set::process_sqlo_set;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use sqlo_set::SqloSetParse;

fn process_all(deriveinput: ::syn::DeriveInput) -> syn::Result<TokenStream> {
    let sqlo: Sqlo = SqloParse::from_derive_input(&deriveinput)?.try_into()?;
    // dbg!(&sqlo);
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
