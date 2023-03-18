use darling::util::IdentString;

use crate::macros::{Assigns, Clauses, Column};

pub trait QueryParser {
    #[cfg(debug_assertions)]
    fn debug(&self) -> bool;
    fn entity(&self) -> &IdentString;
    fn related(&self) -> &Option<IdentString>;
    fn columns(&self) -> &[Column];
    fn assigns(&self) -> &Assigns;
    fn custom_struct(&self) -> &Option<IdentString>;
    fn pk_value(&self) -> &Option<syn::Expr>;
    fn clauses(&self) -> &Clauses;
}
