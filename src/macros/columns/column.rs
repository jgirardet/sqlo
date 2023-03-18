use syn::Token;

use crate::{
    error::SqloError,
    macros::{ColumnToSql, Fragment, Generator},
};

use super::{AliasCast, ColExpr, ColumnCast};

#[derive(Debug, Clone)]
pub enum Column {
    Mono(ColExpr),
    Cast(ColumnCast),
}

impl syn::parse::Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<ColExpr>()?;
        if let ColExpr::Ident(i) = &expr {
            // special case of ident to handle ! and ?
            if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                Ok(Column::Cast(ColumnCast {
                    alias: format!("{i}!").parse()?,
                    expr: i.clone().into(),
                }))
            } else if input.peek(Token![?]) {
                input.parse::<Token![?]>()?;
                Ok(Column::Cast(ColumnCast {
                    alias: format!("{i}?").parse()?,
                    expr: i.clone().into(),
                }))
            } else {
                parse_as(expr, input)
            }
        } else {
            parse_as(expr, input)
        }
    }
}

fn parse_as(expr: ColExpr, input: syn::parse::ParseStream) -> syn::Result<Column> {
    if input.peek(Token![as]) {
        input.parse::<Token![as]>()?;
        let alias = input.parse::<AliasCast>()?;
        Ok(Column::Cast(ColumnCast { expr, alias }))
    } else {
        Ok(Column::Mono(expr))
    }
}

impl ColumnToSql for Column {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError> {
        match self {
            Column::Mono(colexpr) => colexpr.column_to_sql(ctx),
            Column::Cast(colcast) => colcast.column_to_sql(ctx),
        }
    }
}
