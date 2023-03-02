use syn::{BinOp, Token};

use crate::macros::SqlResult;

use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct ColExprOp {
    pub lhs: Box<ColExpr>,
    pub sign: BinOp,
    pub rhs: Box<ColExpr>,
}

impl quote::ToTokens for ColExprOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lhs.to_tokens(tokens);
        self.sign.to_tokens(tokens);
        self.rhs.to_tokens(tokens);
    }
}

impl ColumnToSql for ColExprOp {
    fn column_to_sql(
        &self,
        ctx: &mut SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        let lhs = self.lhs.column_to_sql(ctx)?;
        let sign = self.sign.column_to_sql(ctx)?;
        let rhs = self.rhs.column_to_sql(ctx)?;
        Ok(lhs.add_no_comma(sign).add_no_comma(rhs))
    }
}

impl ColumnToSql for BinOp {
    fn column_to_sql(
        &self,
        _ctx: &mut SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        Ok(op_to_sql(self).to_string().into())
    }
}

pub fn op_to_sql(op: &BinOp) -> &str {
    match op {
        BinOp::Eq(_) => "=",
        BinOp::Ne(_) => "<>",
        BinOp::Le(_) => "<=",
        BinOp::Lt(_) => "<",
        BinOp::Ge(_) => ">=",
        BinOp::Gt(_) => ">",
        // BinOp::And(_) => "AND",
        // BinOp::Or(_) => "||",
        BinOp::Add(_) => "+",
        BinOp::Sub(_) => "-",
        BinOp::Mul(_) => "*",
        BinOp::Div(_) => "/",
        _ => unimplemented!("Sign to str not supported"),
    }
}

pub fn next_is_supported_op(input: &syn::parse::ParseStream) -> bool {
    input.peek(Token![+])
        || input.peek(Token![-])
        || input.peek(Token![*])
        || input.peek(Token![/])
        || input.peek(Token![==])
        || input.peek(Token![!=])
        || input.peek(Token![<=])
        || input.peek(Token![<])
        || input.peek(Token![>])
        || input.peek(Token![>=])
}
