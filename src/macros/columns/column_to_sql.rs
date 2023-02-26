use syn::{Expr, ExprLit, Lit};

use crate::{error::SqloError, macros::SqlQuery, sqlo::Sqlo, sqlos::Sqlos};

pub trait ColumnToSql {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError>;
}

impl ColumnToSql for Lit {
    fn column_to_sql(&self, _main_sqlo: &Sqlo, _sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        let expr: Expr = ExprLit {
            attrs: vec![],
            lit: self.clone(),
        }
        .into();
        Ok(expr.into())
    }
}

impl ColumnToSql for Expr {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        Ok(self.clone().into())
    }
}

// impl ColumnToSql for ExprPath {
//     fn column_to_sql(&self, main_sqlo: &Sqlo, _sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
//         if let Some(ident) = self.path.get_ident() {
//             Ok(main_sqlo.column(ident)?.into())
//         } else {
//             Err(SqloError::new_spanned(self, "Unsupported path expression"))
//         }
//     }
// }

// impl ColumnToSql for ExprField {
//     fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
//         match self.base.as_ref() {
//             Expr::Path(ExprPath { path, .. }) => {
//                 if let Some(ident) = path.get_ident() {
//                     let relation =
//                         sqlos.get_relation(&main_sqlo.ident, &IdentString::new(ident.clone()))?;
//                     let related_sqlo = sqlos.get(&relation.from)?;
//                     if let Member::Named(field_name) = &self.member {
//                         let join = relation.to_inner_join(&sqlos);
//                         let column = related_sqlo.column(&field_name)?;
//                         return Ok((column, join).into());
//                     }
//                     return_error!(
//                         &self.member,
//                         format!("field not found in related [{}]", &related_sqlo.ident)
//                     )
//                 }
//                 return_error!(path, "Unsupported path expression")
//             }
//             _ => Err(SqloError::new_spanned(
//                 self,
//                 "Should be related identifier of a `fk` field",
//             )),
//         }
//     }
// }

// impl ColumnToSql for ExprCall {
//     fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
//         if let Expr::Path(ExprPath { path, .. }) = self.func.as_ref() {
//             if let Some(ident) = path.get_ident() {
//                 let mut args = vec![];
//                 let mut params = vec![];
//                 for arg in self.args.iter() {
//                     args.push(arg.column_to_sql(main_sqlo, sqlos)?);
//                 }
//                 let query = format!("{}({})", ident, args.iter().map(|x| &x.query).join(" ,"));
//                 let mut joins = HashSet::new();
//                 for j in args {
//                     joins.extend(j.joins);
//                     params.extend(j.params);
//                 }
//                 return Ok(SqlQuery {
//                     query,
//                     params,
//                     joins,
//                 });
//             }
//         }
//         return_error!(self, "sql function call must be single word")
//     }
