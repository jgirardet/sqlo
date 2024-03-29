use crate::{
    error::SqloError,
    macros::{ColumnToSql, Context, Generator, Operator},
};

use super::ColExpr;

#[derive(Debug, Clone)]
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
        ctx: &mut Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        ctx.context.push(Context::Operation);
        let lhs = self.lhs.column_to_sql(ctx)?;
        if let ColExpr::Ident(i) = self.rhs.as_ref() {
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
        ctx.context.pop();
        Ok(lhs.add_no_comma(sign).add_no_comma(rhs))
    }
}
