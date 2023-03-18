use crate::{
    error::SqloError,
    macros::{ColumnToSql, Context, Fragment, Generator, Mode, SelectParser},
};
use darling::util::IdentString;
use proc_macro2::TokenStream;
use syn::braced;

#[derive(Debug, Clone)]
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
        ctx: &mut Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        ctx.context.push(Context::SubQuery);
        let parsed = syn::parse2::<SelectParser>(self.tokens.clone()).map_err(SqloError::from)?;
        let result =
            Generator::from_sqlo_query_parse(Mode::Select, parsed, ctx.sqlos, true, ctx.tables.clone())?;
        let mut qr: Fragment = result.try_into()?;
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
