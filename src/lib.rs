mod error;
#[macro_use]
mod utils;
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
use error::SqloError;
use macros::Mode;
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
pub fn select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match Mode::Select.process(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match Mode::Update.process(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
