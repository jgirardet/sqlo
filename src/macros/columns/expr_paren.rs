use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct ColExprParen(Box<ColExpr>);

impl ColExprParen {
    pub fn new(colexpr: ColExpr) -> Self {
        ColExprParen(Box::new(colexpr))
    }
}

impl ColumnToSql for ColExprParen {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        let mut res = self.0.as_ref().column_to_sql(ctx)?;
        res.prepend_str("(");
        res.append_str(")");
        Ok(res)
    }
}

impl quote::ToTokens for ColExprParen {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let content = self.0.as_ref();
        quote::quote![(#content)].to_tokens(tokens);
    }
}
