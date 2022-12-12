use syn::BinOp;

use crate::{
    macros::common::{SelectContext, Sqlize, Sqlized, Validate},
    query_builder::rust_op_to_sql_op,
};

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

impl_trait_to_tokens_for_tokens!(TokenOperator, op);

impl Validate for TokenOperator {}

impl Sqlize for TokenOperator {
    fn sselect(&self, acc: &mut Sqlized, _context: &SelectContext) -> syn::Result<()> {
        acc.append_sql(rust_op_to_sql_op(&self.op).to_string());
        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenOperator {
    fn stry(&self) -> String {
        crate::utils::op_to_str(&self.op).to_string()
    }
}
