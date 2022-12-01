use syn::{Expr, ExprBinary};

use super::SqlToken;

use super::token_operator::TokenOperator;

#[derive(Debug)]
pub struct TokenBinary {
    lhs: Box<SqlToken>,
    op: TokenOperator,
    rhs: Box<SqlToken>,
}

impl_to_tokens_for_tokens!(TokenBinary, lhs, op, rhs);

impl TryFrom<Expr> for TokenBinary {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        if let Expr::Binary(ExprBinary {
            left, op, right, ..
        }) = expr
        {
            return Ok(TokenBinary {
                lhs: Box::new((*left).try_into()?),
                op: op.try_into()?,
                rhs: Box::new((*right).try_into()?),
            });
        }
        return_error!(expr, "invalid expression: not a binary expression")
    }
}
