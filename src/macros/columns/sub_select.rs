use crate::{
    error::SqloError,
    macros::{Context, SqlQuery, SqlResult, SqloSelectParse},
};
use proc_macro2::TokenStream;
use syn::braced;

use super::ColumnToSql;

#[derive(Debug)]
pub struct ColExprSubSelect(TokenStream);

impl quote::ToTokens for ColExprSubSelect {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl ColumnToSql for ColExprSubSelect {
    fn column_to_sql(
        &self,
        ctx: &mut SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        ctx.context.push(Context::SubQuery);
        let parsed = syn::parse2::<SqloSelectParse>(self.0.clone()).map_err(SqloError::from)?;
        let result = SqlResult::from_sqlo_parse(parsed, ctx.sqlos, true)?;
        let mut qr: SqlQuery = result.into();
        qr.prepend_str("(");
        qr.append_str(")");
        ctx.context.pop();
        Ok(qr)
    }
}

impl syn::parse::Parse for ColExprSubSelect {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        Ok(content.parse::<proc_macro2::TokenStream>()?.into())
    }
}

impl From<TokenStream> for ColExprSubSelect {
    fn from(t: TokenStream) -> Self {
        ColExprSubSelect(t)
    }
}
