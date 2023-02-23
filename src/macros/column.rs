use std::collections::HashSet;

use darling::util::IdentString;
use itertools::Itertools;
use syn::{Expr, ExprCall, ExprField, ExprPath, Member};

use crate::{error::SqloError, sqlo::Sqlo, sqlos::Sqlos};

use super::sql_query::SqlQuery;

#[derive(Debug)]
pub enum Column {
    Ident(IdentString),
    Cast(ColumnCast),
    Field(ExprField), //cannot be analysed in parse because we need sqlos
}

impl syn::parse::Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr_base = input.parse::<syn::Expr>()?;
        match expr_base {
            syn::Expr::Cast(syn::ExprCast { expr, ty, .. }) => match ty.as_ref() {
                syn::Type::Path(syn::TypePath { path, .. }) => {
                    if let Some(ident) = path.get_ident() {
                        let alias = IdentString::new(ident.clone());
                        match expr.as_ref() {
                            Expr::Path(_) | Expr::Field(_) | Expr::Call(_) => {
                                return Ok(Column::Cast(ColumnCast { expr: *expr, alias }));
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &expr,
                                    "Not suppored in cast expresssions",
                                ))
                            }
                        }
                    }
                }
                _ => return Err(input.error("Only identifier as cast identifier")),
            },
            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    return Ok(Column::Ident(ident.clone().into()));
                }
            }
            Expr::Field(exprfield) => return Ok(Column::Field(exprfield)),
            _ => {
                return Err(syn::Error::new_spanned(
                    &expr_base,
                    "column's expression should be followed by as",
                ))
            }
        }
        Err(input.error(
            "custom column please use the following: field, related.field or some(expr) as ident",
        ))
    }
}

#[derive(Debug)]
pub struct ColumnCast {
    pub expr: syn::Expr,
    pub alias: IdentString,
}

pub trait ColumnToSql {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError>;
}

impl ColumnToSql for Column {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        match self {
            Column::Ident(ident) => Ok(main_sqlo.column(ident.as_ident())?.into()),
            Column::Cast(colcast) => Ok(colcast.column_to_sql(main_sqlo, sqlos)?),
            Column::Field(exprfield) => Ok(exprfield.column_to_sql(main_sqlo, sqlos)?),
        }
    }
}

impl ColumnToSql for ColumnCast {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        let mut expr = self.expr.column_to_sql(main_sqlo, sqlos)?;
        expr.query = format!("{} as {}", &expr.query, &self.alias);
        Ok(expr)
    }
}

impl ColumnToSql for Expr {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        match self {
            Expr::Path(exprpath) => exprpath.column_to_sql(main_sqlo, sqlos),
            Expr::Field(exprfield) => exprfield.column_to_sql(main_sqlo, sqlos),
            Expr::Call(exprcall) => exprcall.column_to_sql(main_sqlo, sqlos),
            _ => Err(SqloError::new_spanned(self, "Expression not supported")),
        }
    }
}

impl ColumnToSql for ExprPath {
    fn column_to_sql(&self, main_sqlo: &Sqlo, _sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        if let Some(ident) = self.path.get_ident() {
            Ok(main_sqlo.column(ident)?.into())
        } else {
            Err(SqloError::new_spanned(self, "Unsupported path expression"))
        }
    }
}

impl ColumnToSql for ExprField {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        match self.base.as_ref() {
            Expr::Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    let relation =
                        sqlos.get_relation(&main_sqlo.ident, &IdentString::new(ident.clone()))?;
                    let related_sqlo = sqlos.get(&relation.from)?;
                    if let Member::Named(field_name) = &self.member {
                        let join = relation.to_inner_join(&sqlos);
                        let column = related_sqlo.column(&field_name)?;
                        return Ok((column, join).into());
                    }
                    return_error!(
                        &self.member,
                        format!("field not found in related [{}]", &related_sqlo.ident)
                    )
                }
                return_error!(path, "Unsupported path expression")
            }
            _ => Err(SqloError::new_spanned(
                self,
                "Should be related identifier of a `fk` field",
            )),
        }
    }
}

impl ColumnToSql for ExprCall {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        if let Expr::Path(ExprPath { path, .. }) = self.func.as_ref() {
            if let Some(ident) = path.get_ident() {
                let mut args = vec![];
                for arg in self.args.iter() {
                    args.push(arg.column_to_sql(main_sqlo, sqlos)?);
                }
                let query = format!("{}({})", ident, args.iter().map(|x| &x.query).join(" ,"));
                let mut joins = HashSet::new();
                for j in args {
                    joins.extend(j.joins)
                }
                return Ok(SqlQuery {
                    query,
                    params: Vec::default(),
                    joins,
                });
            }
        }
        return_error!(self, "sql function call must be single word")
    }
}
