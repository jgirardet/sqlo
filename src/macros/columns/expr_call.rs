use darling::util::IdentString;

use crate::{
    macros::{ColumnToSql, Context},
    SqloError,
};

use super::ColExprParen;

#[derive(Debug, Clone)]
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
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        if ctx.context.is_empty() {
            // cas in column at begginning of select but not in subquery
            return Err(SqloError::new_spanned(
                self,
                "Call must be followed by `as` with an identifier",
            ));
        }
        // every other cases, as is not mandatory
        ctx.context.push(Context::Call);
        let mut res = self.args.column_to_sql(ctx)?;
        ctx.context.pop();
        res.prepend_str(self.base.as_str());
        Ok(res)
    }
}
