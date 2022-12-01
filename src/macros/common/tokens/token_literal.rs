use syn::{ExprLit, Lit};

#[derive(Debug)]
pub struct TokenLit {
    lit: Lit,
}

impl_to_tokens_for_tokens!(TokenLit, lit);

impl TryFrom<ExprLit> for TokenLit {
    type Error = syn::Error;

    fn try_from(value: ExprLit) -> Result<Self, Self::Error> {
        Ok(TokenLit { lit: value.lit })
    }
}
