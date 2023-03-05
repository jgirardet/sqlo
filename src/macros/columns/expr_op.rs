use crate::{
    error::SqloError,
    macros::{Operator, SqlResult},
};

use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct ColExprOp {
    pub lhs: Box<ColExpr>,
    pub op: Operator,
    pub rhs: Box<ColExpr>,
}

impl quote::ToTokens for ColExprOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lhs.to_tokens(tokens);
        self.op.to_tokens(tokens);
        self.rhs.to_tokens(tokens);
    }
}

impl ColumnToSql for ColExprOp {
    fn column_to_sql(
        &self,
        ctx: &mut SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        let lhs = self.lhs.column_to_sql(ctx)?;
        if let ColExpr::Ident(i) = self.rhs.as_ref() {
            dbg!(&self.rhs);
            if i.as_str() == "None" {
                match &self.op {
                    Operator::Eq => {
                        return Ok(self.lhs.column_to_sql(ctx)?.add_no_comma(" IS NULL".into()))
                    }

                    Operator::Neq => {
                        return Ok(self
                            .lhs
                            .column_to_sql(ctx)?
                            .add_no_comma(" IS NOT NULL".into()))
                    }
                    _ => {
                        return Err(SqloError::new_spanned(
                            self.op,
                            "None must be used with == or !=",
                        ))
                    }
                }
            }
        };
        let sign = self.op.column_to_sql(ctx)?;
        let rhs = self.rhs.column_to_sql(ctx)?;
        Ok(lhs.add_no_comma(sign).add_no_comma(rhs))
    }
}
