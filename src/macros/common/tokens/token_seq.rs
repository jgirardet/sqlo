use syn::{punctuated::Punctuated, Expr, Token};

use crate::macros::common::{FromContext, SelectContext, Sqlize, Sqlized, Validate};

use super::SqlToken;

#[derive(Debug)]
pub struct TokenSeq {
    content: Punctuated<SqlToken, Token!(,)>,
}

impl<'a> TokenSeq {
    pub fn iter(&'a self) -> syn::punctuated::Iter<'a, SqlToken> {
        self.content.iter()
    }
}

impl IntoIterator for TokenSeq {
    type Item = SqlToken;

    type IntoIter = syn::punctuated::IntoIter<SqlToken>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

impl<'a> IntoIterator for &'a TokenSeq {
    type Item = &'a SqlToken;

    type IntoIter = syn::punctuated::Iter<'a, SqlToken>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.iter()
    }
}

impl_trait_to_tokens_for_tokens!(TokenSeq, content);

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

impl Validate for TokenSeq {
    fn validate(&self, sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
        for it in &self.content {
            it.validate(sqlos)?
        }
        Ok(())
    }
}

impl Sqlize for TokenSeq {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        let mut group = Sqlized::default();
        context.lower();
        let level = context.query_context.clone();
        for t in self.iter() {
            context.query_context = level.clone(); // reinit query context for each element
            t.sselect(&mut group, context)?;
        }
        acc.append_group_with(group, ",");
        Ok(())
    }

    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        let mut group = Sqlized::default();
        for t in self.iter() {
            match t {
                SqlToken::Cast(x) => x.ffrom(&mut group, context)?,
                SqlToken::Ident(x) => x.ffrom(&mut group, context)?,
                _ => return_error!(&t, "Unimplemented in From context"),
            }
        }
        acc.append_group_with(group, ",");
        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenSeq {
    fn stry(&self) -> String {
        use itertools::Itertools;
        self.content.iter().map(|x| x.stry()).join(",")
    }
}
