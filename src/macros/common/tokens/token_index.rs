use syn::{Expr, ExprIndex};

use crate::macros::common::Validate;

use super::{SqlToken, TokenIdent};
use crate::macros::common::{SelectContext, Sqlize, Sqlized};

#[derive(Debug)]
pub struct TokenIndex {
    name: TokenIdent,
    content: Box<SqlToken>,
}

impl_trait_to_tokens_for_tokens!(TokenIndex, name, content);

impl TryFrom<Expr> for TokenIndex {
    type Error = syn::Error;

    fn try_from(parent: Expr) -> Result<Self, Self::Error> {
        if let Expr::Index(ExprIndex { expr, index, .. }) = parent {
            return Ok(TokenIndex {
                name: (*expr).try_into()?,
                content: Box::new((*index).try_into()?),
            });
        }
        return_error!(parent, "invalid input, not an index expression")
    }
}

impl Validate for TokenIndex {}

impl Sqlize for TokenIndex {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        let mut group = Sqlized::default();

        // Small hack to handle distinct in TokenCall
        if self.name.as_str() == "DISTINCT" {
            group.append_sql("DISTINCT ".to_string());
            self.content.sselect(&mut group, context)?;
        } else {
            unimplemented!("indexing is not implemented yet")
        }
        acc.append_group(group);

        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenIndex {
    fn stry(&self) -> String {
        if self.name.as_str() == "DISTINCT" {
            format!("DISTINCT {}", self.content.as_ref().stry(),)
        } else {
            format!("")
        }
    }
}
