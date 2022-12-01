use crate::macros::common::keyword::{kw, SqlKeyword};

use super::{token_ident::TokenIdent, SqlToken};

#[derive(Debug)]
pub struct TokenCast {
    initial: Box<SqlToken>,
    alias: TokenIdent,
    sep: CastSeparator,
}

impl TokenCast {
    pub fn new(initial: SqlToken, alias: TokenIdent, sep: CastSeparator) -> TokenCast {
        TokenCast {
            initial: Box::new(initial),
            alias,
            sep,
        }
    }
}

impl_to_tokens_for_tokens!(TokenCast, initial, sep, alias);

#[derive(Debug)]
pub enum CastSeparator {
    None,
    AS(SqlKeyword),
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
// unused right now, parse via sqltoken
//
// impl syn::parse::Parse for TokenCast {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let initial = input.parse::<Expr>()?;
//         let sep = if input.peek(kw::AS) {
//             input.parse::<kw::AS>()?;
//             CastSeparator::AS
//         } else {
//             CastSeparator::None
//         };
//         let alias = input.parse::<TokenIdent>()?;
//         Ok(TokenCast {
//             initial: Box::new(initial.try_into()?),
//             alias,
//             sep,
//         })
//     }
// }
