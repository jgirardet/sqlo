use super::SqlToken;

pub type SqlTokens<'a> = syn::punctuated::Iter<'a, SqlToken>;
