use darling::util::IdentString;
use syn::{Expr, ExprField, ExprPath, Member};

use crate::{error::SqloError, sqlo::Sqlo, sqlos::Sqlos};

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
                        return Ok(Column::Cast(ColumnCast { expr: *expr, alias }));
                    }
                }
                _ => {}
            },
            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    return Ok(Column::Ident(ident.clone().into()));
                }
            }
            Expr::Field(exprfield) => return Ok(Column::Field(exprfield)),
            _ => return Err(input.error("column's expression should be followed by as")),
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
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<String, SqloError>;
}

impl ColumnToSql for Column {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<String, SqloError> {
        match self {
            Column::Ident(ident) => main_sqlo.column(ident.as_ident()),
            Column::Cast(colcast) => colcast.column_to_sql(main_sqlo, sqlos),
            Column::Field(exprfield) => exprfield.column_to_sql(main_sqlo, sqlos),
        }
    }
}

impl ColumnToSql for ColumnCast {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<String, SqloError> {
        let expr = self.expr.column_to_sql(main_sqlo, sqlos)?;
        Ok(format!("{expr} as {}", &self.alias))
    }
}

impl ColumnToSql for Expr {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<String, SqloError> {
        match self {
            Expr::Path(exprpath) => exprpath.column_to_sql(main_sqlo, sqlos),
            _ => Err(SqloError::new_spanned(self, "Expression not supported")),
        }
    }
}

impl ColumnToSql for ExprPath {
    fn column_to_sql(&self, main_sqlo: &Sqlo, _sqlos: &Sqlos) -> Result<String, SqloError> {
        if let Some(ident) = self.path.get_ident() {
            main_sqlo.column(ident)
        } else {
            Err(SqloError::new_spanned(self, "Unsupported path expression"))
        }
    }
}

impl ColumnToSql for ExprField {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<String, SqloError> {
        match self.base.as_ref() {
            Expr::Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    let related_sqlo = sqlos
                        .get_by_relation(&main_sqlo.ident, &IdentString::new(ident.clone()))?;
                    if let Member::Named(field_name) = &self.member {
                        return related_sqlo.column(&field_name);
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
