use syn::{punctuated::Punctuated, Expr, Token};

use super::SqlToken;

#[derive(Debug)]
pub struct TokenSeq {
    content: Punctuated<SqlToken, Token!(,)>,
}

impl_to_tokens_for_tokens!(TokenSeq, content);

impl TryFrom<Punctuated<Expr, Token![,]>> for TokenSeq {
    type Error = syn::Error;

    fn try_from(punctuated: Punctuated<Expr, Token![,]>) -> Result<Self, Self::Error> {
        let mut content = Punctuated::<SqlToken, Token!(,)>::new();
        for v in punctuated.into_iter() {
            content.push(v.try_into()?);
        }
        Ok(TokenSeq { content })
    }
}

impl syn::parse::Parse for TokenSeq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let res = Punctuated::parse_separated_nonempty(input)?;
        Ok(TokenSeq { content: res })
    }
}

impl From<Punctuated<SqlToken, Token!(,)>> for TokenSeq {
    fn from(p: Punctuated<SqlToken, Token!(,)>) -> Self {
        TokenSeq { content: p }
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenSeq {
    fn stry(&self) -> String {
        use itertools::Itertools;
        self.content.iter().map(|x| x.stry()).join(",")
    }
}
