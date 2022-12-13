use syn::{Expr, ExprParen, ExprTuple};

use crate::macros::common::Validate;

use super::SqlToken;
use crate::macros::common::{SelectContext, Sqlize, Sqlized};

#[derive(Debug)]
pub struct TokenParen {
    content: Box<SqlToken>,
}

impl_trait_to_tokens_for_tokens!(TokenParen, content);

impl TryFrom<Expr> for TokenParen {
    type Error = syn::Error;

    fn try_from(parent: Expr) -> Result<Self, Self::Error> {
        if let Expr::Paren(ExprParen { expr, .. }) = parent {
            return Ok(TokenParen {
                content: Box::new((*expr).try_into()?),
            });
        } else if let Expr::Tuple(ExprTuple { elems, .. }) = parent {
            return Ok(TokenParen {
                content: Box::new(elems.try_into()?),
            });
        }
        return_error!(parent, "invalid input, not a parenthes expression")
    }
}

impl Validate for TokenParen {}

impl Sqlize for TokenParen {
    fn sselect(&self, acc: &mut Sqlized, context: &SelectContext) -> syn::Result<()> {
        if let SqlToken::ExprSeq(_) = self.content.as_ref() {
            return_error!(
                &self.content,
                "Comma separated values not allowed inside parenthesis"
            )
        }
        let mut group = Sqlized::default();
        group.append_sql("(".to_string());
        self.content.sselect(&mut group, context)?;
        group.append_sql(")".to_string());
        acc.append_group(group);

        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenParen {
    fn stry(&self) -> String {
        format!("({})", self.content.as_ref().stry(),)
    }
}
