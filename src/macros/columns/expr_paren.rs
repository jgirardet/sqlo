use syn::{parenthesized, punctuated::Punctuated, Token};

use crate::{
    error::SqloError,
    macros::{Context, Fragment, ColumnToSql},
};

use super::ColExpr;


#[derive(Debug)]
pub struct ColExprParen(Punctuated<ColExpr, Token![,]>);

impl ColumnToSql for ColExprParen {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        ctx.context.push(Context::Paren);
        let mut res = self.0.iter().fold(
            Ok(Fragment::default()),
            |acc: Result<Fragment, SqloError>, nex| Ok(acc.unwrap() + nex.column_to_sql(ctx)?),
        )?;
        res.prepend_str("(");
        res.append_str(")");
        ctx.context.pop();
        Ok(res)
    }
}

impl quote::ToTokens for ColExprParen {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<Punctuated<ColExpr, Token![,]>> for ColExprParen {
    fn from(p: Punctuated<ColExpr, Token![,]>) -> Self {
        ColExprParen(p)
    }
}

impl syn::parse::Parse for ColExprParen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let seq: Punctuated<ColExpr, Token![,]> = Punctuated::parse_separated_nonempty(&content)?;
        Ok(seq.into())
    }
}
