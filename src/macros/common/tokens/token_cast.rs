use std::fmt::Display;

use super::SqlToken;
use crate::macros::common::{
    keyword::{kw, SqlKeyword},
    FromContext, SelectContext, Sqlize, Sqlized, Validate,
};

#[derive(Debug)]
pub struct TokenCast {
    pub initial: Box<SqlToken>,
    pub alias: Box<SqlToken>,
    pub sep: CastSeparator,
}

impl TokenCast {
    pub fn new(initial: SqlToken, alias: SqlToken, sep: CastSeparator) -> TokenCast {
        TokenCast {
            initial: Box::new(initial),
            alias: Box::new(alias),
            sep,
        }
    }
}

impl_trait_to_tokens_for_tokens!(TokenCast, initial, sep, alias);

#[derive(Debug)]
pub enum CastSeparator {
    None, //eqal a space
    AS(SqlKeyword),
}

impl Display for CastSeparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AS(_) => write!(f, "AS"),
            Self::None => write!(f, ""),
        }
    }
}

impl TryFrom<SqlKeyword> for CastSeparator {
    type Error = syn::Error;

    fn try_from(value: SqlKeyword) -> Result<Self, Self::Error> {
        match value {
            SqlKeyword::AS(_) => Ok(CastSeparator::AS(value)),
            _ => return_error!(value, "Keyword not usable with cast"),
        }
    }
}

impl quote::ToTokens for CastSeparator {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::AS(s) => s.to_tokens(tokens),
            Self::None => {}
        }
    }
}

impl syn::parse::Parse for CastSeparator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::AS) {
            Ok(input.parse::<SqlKeyword>()?.try_into()?)
        } else {
            Ok(CastSeparator::None)
        }
    }
}

impl Sqlize for TokenCast {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        context.lower();
        let mut group = Sqlized::default();
        self.initial.sselect(&mut group, context)?;
        group.append_sql(self.sep.to_string());

        match self.alias.as_ref() {
            SqlToken::Literal(l) => l.sselect(&mut group, context)?,
            SqlToken::Ident(i) => group.append_sql(i.to_string()),
            _ => return_error!(&self.alias, "should be identifier ou string"),
        };
        acc.append_sql(group.sql().join(" "));
        Ok(())
    }

    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        let mut group = Sqlized::default();
        self.initial.ffrom(&mut group, context)?;
        match self.alias.as_ref() {
            SqlToken::Ident(i) => group.append_sql(i.to_string()),
            _ => return_error!(&self.alias, "Should be a identifier"),
        }
        acc.append_sql(group.sql().join(" "));
        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for CastSeparator {
    fn stry(&self) -> String {
        match self {
            Self::AS(_) => " AS ".to_string(),
            Self::None => " ".to_string(),
        }
    }
}

impl Validate for TokenCast {
    fn validate(&self, sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
        self.initial.validate(sqlos)
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenCast {
    fn stry(&self) -> String {
        format!(
            "{}{}{}",
            self.initial.as_ref().stry(),
            self.sep.stry(),
            self.alias.stry()
        )
    }
}
