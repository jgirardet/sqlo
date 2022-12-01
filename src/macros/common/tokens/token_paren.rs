use syn::{Expr, ExprParen};

use super::SqlToken;

#[derive(Debug)]
pub struct TokenParen {
    content: Box<SqlToken>,
}

impl_to_tokens_for_tokens!(TokenParen, content);

impl TryFrom<Expr> for TokenParen {
    type Error = syn::Error;

    fn try_from(parent: Expr) -> Result<Self, Self::Error> {
        if let Expr::Paren(ExprParen { expr, .. }) = parent {
            return Ok(TokenParen {
                content: Box::new((*expr).try_into()?),
            });
        }
        return_error!(parent, "invalid input, not a parenthes expression")
    }
}
