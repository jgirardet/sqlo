use darling::util::IdentString;
use syn::{Expr, ExprLit, Lit};

use crate::{
    error::SqloError,
    macros::{Context, SqlQuery, SqlResult},
};

pub trait ColumnToSql {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError>;
}

impl ColumnToSql for Lit {
    fn column_to_sql(&self, _ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        let expr: Expr = ExprLit {
            attrs: vec![],
            lit: self.clone(),
        }
        .into();
        Ok(expr.into())
    }
}

impl ColumnToSql for Expr {
    fn column_to_sql(&self, _ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        Ok(self.clone().into())
    }
}

impl ColumnToSql for &IdentString {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        if ctx.alias.contains_key(self) {
            if ctx.context.contains(&Context::Call) {
                Ok(format! {"{self}"}.into()) // no sqlx text alias in call
            } else {
                Ok(ctx.alias[self].clone().into())
            }
        } else {
            Ok(ctx.main_sqlo.column(self.as_ident())?.into())
        }
    }
}
