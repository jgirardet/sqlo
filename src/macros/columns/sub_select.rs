use crate::{
    error::SqloError,
    macros::{Context, Mode, SqlQuery, SqlResult, SqloQueryParse},
};
use darling::util::IdentString;
use proc_macro2::TokenStream;
use syn::braced;

use super::ColumnToSql;

#[derive(Debug)]
pub struct ColExprSubSelect {
    tokens: TokenStream,
    func: Option<IdentString>,
}

impl quote::ToTokens for ColExprSubSelect {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.func.to_tokens(tokens);
        self.tokens.to_tokens(tokens);
    }
}

impl ColumnToSql for ColExprSubSelect {
    fn column_to_sql(
        &self,
        ctx: &mut SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        ctx.context.push(Context::SubQuery);
        let parsed = syn::parse2::<SqloQueryParse>(self.tokens.clone()).map_err(SqloError::from)?;
        let result =
            SqlResult::from_sqlo_parse(Mode::Select, parsed, ctx.sqlos, true, ctx.table_aliases())?;
        let mut qr: SqlQuery = result.try_into()?;
        qr.prepend_str("(");
        if let Some(func) = &self.func {
            qr.prepend_str(func.as_str())
        }
        qr.append_str(")");
        ctx.context.pop();
        Ok(qr)
    }
}

impl ColExprSubSelect {
    pub fn parse_with_ident(
        ident: IdentString,
        input: syn::parse::ParseStream,
    ) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self {
            tokens: content.parse::<proc_macro2::TokenStream>()?,
            func: Some(ident),
        })
    }

    pub fn parse_without_ident(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self {
            tokens: content.parse::<proc_macro2::TokenStream>()?,
            func: None,
        })
    }
}
