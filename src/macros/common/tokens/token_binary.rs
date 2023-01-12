use syn::{Expr, ExprBinary};

use crate::macros::common::{SelectContext, Sqlize, Sqlized, Validate};

use super::SqlToken;

use super::token_operator::TokenOperator;

#[derive(Debug)]
pub struct TokenBinary {
    lhs: Box<SqlToken>,
    op: TokenOperator,
    rhs: Box<SqlToken>,
}

impl_trait_to_tokens_for_tokens!(TokenBinary, lhs, op, rhs);

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

impl Validate for TokenBinary {}

impl Sqlize for TokenBinary {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        let mut group = Sqlized::default();
        self.lhs.sselect(&mut group, context)?;
        self.op.sselect(&mut group, context)?;
        self.rhs.sselect(&mut group, context)?;
        acc.append_group_with(group, " ");
        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenBinary {
    fn stry(&self) -> String {
        format!(
            "{} {} {}",
            self.lhs.as_ref().stry(),
            &self.op.stry(),
            &self.rhs.as_ref().stry()
        )
    }
}
