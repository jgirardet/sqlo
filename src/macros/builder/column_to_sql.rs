use darling::util::IdentString;
use syn::{Expr, ExprLit, Lit};

use crate::{
    error::SqloError,
    macros::{Context, Fragment, Generator},
};

pub trait ColumnToSql {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError>;
}

impl ColumnToSql for Lit {
    fn column_to_sql(&self, _ctx: &mut Generator) -> Result<Fragment, SqloError> {
        let expr: Expr = ExprLit {
            attrs: vec![],
            lit: self.clone(),
        }
        .into();
        Ok(expr.into())
    }
}

impl ColumnToSql for Expr {
    fn column_to_sql(&self, _ctx: &mut Generator) -> Result<Fragment, SqloError> {
        Ok(self.clone().into())
    }
}

impl ColumnToSql for &IdentString {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError> {
        // only the aliases
        if ctx.aliases.contains_key(self) {
            if ctx.context.contains(&Context::Call) {
                Ok(format! {"{self}"}.into()) // no sqlx text alias in call
            } else {
                Ok(ctx.aliases[self].clone().into())
            }
        } else {
            // all ident from main sqlo
            Ok(ctx.column(&ctx.main_sqlo.ident, self)?.into())
        }
    }
}
