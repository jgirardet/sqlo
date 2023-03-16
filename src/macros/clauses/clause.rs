use crate::error::SqloError;
use syn::parse::ParseStream;

use super::{GroupBy, Having, Limit, OrderBy, Where};

#[derive(Debug, Clone)]
pub enum Clause {
    Where(Where),
    GroupBy(GroupBy),
    Having(Having),
    Limit(Limit),
    OrderBy(OrderBy),
}

macro_rules! impl_from_from_clause {
    ($($variant:ident),+) => {
        $(
        impl From<$crate::macros::$variant> for Clause {
            fn from(variant: $crate::macros::$variant) -> Clause {
                Clause::$variant(variant)
            }
        }
    )+
    };
}

impl_from_from_clause! { Where, GroupBy, Having, Limit, OrderBy }

#[derive(Debug, Clone)]
pub struct Clauses(Vec<Clause>);

impl Clauses {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn try_push<F>(&mut self, input: ParseStream, f: F) -> Result<(), SqloError>
    where
        Self: Sized,
        F: FnOnce(ParseStream) -> syn::Result<Option<Clause>>,
    {
        if let Some(opt) = f(input)? {
            self.0.push(opt)
        }
        Ok(())
    }

    pub fn iter(&self) -> std::slice::Iter<Clause> {
        self.0.iter()
    }
}
