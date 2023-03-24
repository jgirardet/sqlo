use darling::util::IdentString;

use crate::macros::{Assigns, Clauses, Column};

use super::Fetch;

pub trait QueryParser {
    #[cfg(debug_assertions)]
    fn debug(&self) -> bool;
    fn entity(&self) -> &IdentString;
    fn related(&self) -> &Option<IdentString>;
    fn columns(&self) -> &[Column];
    fn assigns(&self) -> &Assigns;
    fn custom_struct(&self) -> Option<IdentString>;
    fn pk_value(&self) -> PkValue;
    fn clauses(&self) -> &Clauses;
    fn fetch(&self) -> Fetch;
}

#[derive(Debug, Clone)]
pub enum PkValue {
    Parenthezide(syn::Expr),
    Bracketed(syn::Expr),
    None,
}
