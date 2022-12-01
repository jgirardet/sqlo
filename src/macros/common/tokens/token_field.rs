use syn::{Expr, Member};

use super::token_ident::TokenIdent;

#[derive(Debug)]
pub struct TokenField {
    base: TokenIdent,
    member: TokenIdent,
}

impl_to_tokens_for_tokens!(TokenField, base, member);

impl TryFrom<Expr> for TokenField {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        if let Expr::Field(ref exprfield) = expr {
            if let Expr::Path(p) = exprfield.base.as_ref() {
                let base: TokenIdent = p.try_into()?;
                if let Member::Named(ref i) = exprfield.member {
                    return Ok(TokenField {
                        base,
                        member: i.into(),
                    });
                }
            }
        }
        return_error!(
            expr,
            "Invalid expression: should be sqlo_struct.fieldname or alias.fieldname"
        )
    }
}
