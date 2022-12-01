use syn::BinOp;

#[derive(Debug)]
pub struct TokenOperator {
    op: BinOp,
}

impl TryFrom<BinOp> for TokenOperator {
    type Error = syn::Error;
    fn try_from(op: BinOp) -> syn::Result<Self> {
        Ok(TokenOperator { op })
    }
}

impl_to_tokens_for_tokens!(TokenOperator, op);
