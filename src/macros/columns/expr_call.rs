use darling::util::IdentString;

use crate::macros::Context;

use super::{ColExprParen, ColumnToSql};

#[derive(Debug)]
pub struct ColExprCall {
    pub base: IdentString,
    pub args: ColExprParen,
}

impl quote::ToTokens for ColExprCall {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.base.to_tokens(tokens);
        self.args.to_tokens(tokens);
    }
}

impl ColumnToSql for ColExprCall {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        ctx.context = Context::Call;
        let mut res = self.args.column_to_sql(ctx)?;
        ctx.context = Context::None;
        res.prepend_str(self.base.as_str());
        Ok(res)
    }
}
