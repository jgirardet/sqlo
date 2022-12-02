use syn::{Expr, ExprCall};

use super::{token_seq::TokenSeq, TokenIdent};

#[derive(Debug)]
pub struct TokenCall {
    func: TokenIdent,
    content: TokenSeq,
}

impl_to_tokens_for_tokens!(TokenCall, func, content);

impl TryFrom<Expr> for TokenCall {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        if let Expr::Call(ExprCall { func, args, .. }) = expr {
            return Ok(TokenCall {
                func: (*func).try_into()?,
                content: args.try_into()?,
            });
        }
        return_error!(expr, "invalid expression: not a call expression")
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenCall {
    fn stry(&self) -> String {
        format!("{}({})", &self.func.stry(), &self.content.stry())
    }
}
