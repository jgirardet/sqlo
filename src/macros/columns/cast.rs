use std::fmt::Display;

use darling::util::IdentString;
use syn::LitStr;

use crate::{
    error::SqloError,
    macros::{SqlQuery, SqlResult},
};

use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct ColumnCast {
    pub expr: ColExpr,
    pub alias: AliasCast,
}

impl ColumnToSql for ColumnCast {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        let mut expr = self.expr.column_to_sql(ctx)?;
        expr.query = format!("{} as {}", &expr.query, &self.alias);
        Ok(expr)
    }
}

#[derive(Debug)]
pub enum AliasCast {
    Ident(IdentString),
    Literal(LitStr),
}

impl From<&syn::Ident> for AliasCast {
    fn from(ident: &syn::Ident) -> Self {
        AliasCast::Ident(IdentString::new(ident.clone()))
    }
}

impl Display for AliasCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AliasCast::Ident(i) => write!(f, "{}", i),
            AliasCast::Literal(l) => write!(f, "\"{}\"", l.value()),
        }
    }
}

impl syn::parse::Parse for AliasCast {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input.parse::<LitStr>() {
            Ok(l) => Ok(AliasCast::Literal(l)),
            Err(_) => match input.parse::<syn::Ident>() {
                Ok(i) => Ok(AliasCast::Ident(i.into())),
                Err(_) => Err(input.error("as must be followed by identifier or string literal")),
            },
        }
    }
}
