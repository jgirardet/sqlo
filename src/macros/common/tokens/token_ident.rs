use std::fmt::Display;

use darling::util::IdentString;
use syn::{Expr, ExprPath, Ident};

use crate::macros::common::{
    FromContext, QueryContext, QueryMoment, SelectContext, Sqlize, Sqlized, Validate,
};

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct TokenIdent {
    ident: IdentString,
}

impl AsRef<IdentString> for TokenIdent {
    fn as_ref(&self) -> &IdentString {
        &self.ident
    }
}

impl From<Ident> for TokenIdent {
    fn from(ident: Ident) -> Self {
        TokenIdent {
            ident: ident.into(),
        }
    }
}

impl TokenIdent {
    pub fn as_str(&self) -> &str {
        self.ident.as_str()
    }
    pub fn as_ident(&self) -> &Ident {
        self.ident.as_ident()
    }
}

impl_trait_to_tokens_for_tokens!(TokenIdent, ident);

impl syn::parse::Parse for TokenIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        Ok(TokenIdent {
            ident: ident.into(),
        })
    }
}

impl From<&Ident> for TokenIdent {
    fn from(ident: &Ident) -> Self {
        TokenIdent {
            ident: ident.clone().into(),
        }
    }
}

impl TryFrom<&ExprPath> for TokenIdent {
    type Error = syn::Error;
    fn try_from(p: &ExprPath) -> Result<Self, Self::Error> {
        if let Some(ident) = p.path.get_ident() {
            Ok(ident.into())
        } else {
            return_error!(p, "invalid column identifier")
        }
    }
}

impl TryFrom<Expr> for TokenIdent {
    type Error = syn::Error;
    fn try_from(p: Expr) -> Result<Self, Self::Error> {
        if let Expr::Path(ref expr_path) = p {
            return expr_path.try_into();
        }
        return_error!(p, "invalid column identifier")
    }
}

impl Display for TokenIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)
    }
}

impl PartialEq<IdentString> for TokenIdent {
    fn eq(&self, other: &IdentString) -> bool {
        &self.ident == other
    }
}

impl PartialEq<TokenIdent> for IdentString {
    fn eq(&self, other: &TokenIdent) -> bool {
        self == &other.ident
    }
}

impl PartialEq<TokenIdent> for TokenIdent {
    fn eq(&self, other: &TokenIdent) -> bool {
        &self.ident == other
    }
}

impl Validate for TokenIdent {}

impl Sqlize for TokenIdent {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        let mut testes = vec![];
        for alias_sqlo in context.alias_sqlos.iter() {
            if let Some(field) = alias_sqlo.sqlo().field(self.as_str()) {
                match context.query_context {
                    QueryContext::SqloAs(QueryMoment::InClause) => {
                        acc.append_sql(field.as_query.to_string());
                    }
                    _ => {
                        acc.append_sql(field.column.to_string());
                    }
                };
                return Ok(());
            } else {
                testes.push(&alias_sqlo.sqlo().ident)
            }
        }
        return_error!(
            self,
            format!("Field not found in [{}]", testes.iter().join(","))
        )
    }

    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        match context
            .alias_sqlos
            .iter()
            .find(|x| x.sqlo().ident == self.ident)
        {
            Some(alias) => {
                acc.append_sql(alias.sqlo().tablename.to_string());
                return Ok(());
            }
            None => {
                return_error!(&self, "No valid Sqlo struct in clause FROM") //maybe unreachable
            }
        }
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenIdent {
    fn stry(&self) -> String {
        self.ident.to_string()
    }
}
