use crate::{
    macros::common::{
        AliasSqlo, AliasSqlos, SqlKeyword, SqlToken, SqlTokens, Sqlize, Sqlized, ToAliasSqlos,
        TokenSeq,
    },
    sqlos::Sqlos,
};

#[derive(Debug)]
pub struct ClauseFrom {
    keyword: SqlKeyword,
    tokens: TokenSeq,
}

impl_from_for_clause_variant!(ClauseFrom  From FROM);
impl_validate_for_clause_variant!(ClauseFrom);
impl_parse_for_clause!(ClauseFrom FROM);
impl_stry_for_clause!(ClauseFrom "FROM");
impl_trait_to_tokens_for_clause!(ClauseFrom, tokens);
impl_clause_trait_for_clause_variant!(ClauseFrom);

// impl<'a> ClauseFrom {
//     pub fn to_alias_sqlos(&'a self, sqlos: &'a Sqlos) -> syn::Result<Vec<AliasSqlo>> {
//         let mut res = vec![];
//         for t in &self.tokens {
//             match t {
//                 SqlToken::Cast(token_cast) => {
//                     if let SqlToken::Ident(ref ti) = *token_cast.initial {
//                         let sqlo = sqlos.get(ti.as_str())?;
//                         match token_cast.alias.as_ref() {
//                             SqlToken::Ident(i) => res.push(AliasSqlo {
//                                 sqlo,
//                                 alias: Some(i),
//                             }),
//                             _ => return_error!(&token_cast.alias, "Should be identifier"),
//                         }
//                     }
//                 }
//                 SqlToken::Ident(ti) => {
//                     let sqlo = sqlos.get(ti.as_str())?;
//                     res.push(AliasSqlo { sqlo, alias: None });
//                 }
//                 _ => unreachable!("From clause has only Cast and Ident variant"),
//             }
//         }
//         Ok(res)
//     }
// }

impl Sqlize for ClauseFrom {
    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        self.keyword.ffrom(acc, context)?;
        self.tokens.ffrom(acc, context)?;
        Ok(())
    }
}

impl<'a, 'b> ToAliasSqlos<'a, 'b> for ClauseFrom {}

#[derive(Debug)]
pub struct FromContext<'a, 'b> {
    pub alias_sqlos: &'a AliasSqlos<'a, 'b>,
}

impl<'a, 'b> FromContext<'a, 'b> {
    pub fn from_clausefrom(alias_sqlos: &'a AliasSqlos<'a, 'b>) -> syn::Result<Self> {
        // let alias_sqlos = clause.to_alias_sqlos(sqlos)?;
        Ok(Self { alias_sqlos })
    }
}
