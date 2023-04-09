use darling::util::IdentString;
use quote::ToTokens;
use syn::{Expr, ExprLit, Lit};

use crate::{
    error::SqloError,
    macros::{ColExpr, Context, Fragment, Generator},
};

use super::Mode;

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
        // if ctx.aliases.contains_key(self) {
            // if ctx.context.contains(&Context::Call) {
                // Ok(format! {"{self}"}.into()) // no sqlx text alias in call
            // } else {
            //     Ok(ctx.aliases[self].clone().into())
            // }
        // } else {
            // all ident from main sqlo
            match ctx.mode {
                Mode::Select => Ok(ctx
                    .tables
                    .alias_dot_column(&ctx.main_sqlo.ident, self)?
                    .into()),
                _ => {
                    // convert ident to value if no column
                    if ctx.main_sqlo.field(self.as_ident()).is_none() {
                        return ColExpr::Value(syn::parse2::<syn::Expr>(self.to_token_stream())?)
                            .column_to_sql(ctx);
                    }

                    Ok(ctx.tables.column(&ctx.main_sqlo.ident, self)?.into())
                }
            }
        // }
    }
}
