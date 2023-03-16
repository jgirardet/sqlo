use darling::util::IdentString;

use crate::macros::{Column, Clauses};

pub trait QueryParser {
    #[cfg(debug_assertions)]
    fn debug(&self) -> bool;
    fn entity(&self) -> &IdentString;
    fn related(&self) -> &Option<IdentString>;
    fn columns(&self) -> &[Column];
    fn custom_struct(&self) -> &Option<IdentString>;
    fn pk_value(&self) -> &Option<syn::Expr>;
    fn clauses(&self) -> &Clauses;
}
