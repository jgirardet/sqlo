use syn::Token;

use crate::{
    error::SqloError,
    macros::{SqlQuery, SqlResult},
};

use super::{AliasCast, ColExpr, ColumnCast, ColumnToSql};

#[derive(Debug)]
pub enum Column {
    Mono(ColExpr),
    Cast(ColumnCast),
}

impl syn::parse::Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<ColExpr>()?;
        if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let alias = input.parse::<AliasCast>()?;
            Ok(Column::Cast(ColumnCast { expr, alias }))
        } else {
            Ok(Column::Mono(expr))
        }
    }
}

impl ColumnToSql for Column {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        match self {
            Column::Mono(colexpr) => colexpr.column_to_sql(ctx),
            Column::Cast(colcast) => colcast.column_to_sql(ctx),
        }
    }
}
