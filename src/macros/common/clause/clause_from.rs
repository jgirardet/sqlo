use darling::util::IdentString;

use crate::{
    macros::common::{SqlKeyword, SqlToken, Sqlize, Sqlized, TokenIdent, TokenSeq},
    sqlo::Sqlo,
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

impl<'a> ClauseFrom {
    pub fn to_alias_sqlos(&'a self, sqlos: &'a Sqlos) -> syn::Result<Vec<AliasSqlo>> {
        let mut res = vec![];
        for t in &self.tokens {
            match t {
                SqlToken::Cast(token_cast) => {
                    if let SqlToken::Ident(ref ti) = *token_cast.initial {
                        let sqlo = sqlos.get(ti.as_str())?;
                        match token_cast.alias.as_ref() {
                            SqlToken::Ident(i) => res.push(AliasSqlo {
                                sqlo,
                                alias: Some(i),
                            }),
                            _ => return_error!(&token_cast.alias, "Should be identifier"),
                        }
                    }
                }
                SqlToken::Ident(ti) => {
                    let sqlo = sqlos.get(ti.as_str())?;
                    res.push(AliasSqlo { sqlo, alias: None });
                }
                _ => unreachable!("From clause has only Cast and Ident variant"),
            }
        }
        Ok(res)
    }
}

impl Sqlize for ClauseFrom {
    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        self.keyword.ffrom(acc, context)?;
        self.tokens.ffrom(acc, context)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AliasSqlo<'a> {
    pub sqlo: &'a Sqlo,
    pub alias: Option<&'a TokenIdent>,
}

impl<'a> AliasSqlo<'a> {
    pub fn ident(&self) -> &'a IdentString {
        &self.sqlo.ident
    }
}

#[derive(Debug)]
pub struct FromContext<'a> {
    pub alias_sqlos: Vec<AliasSqlo<'a>>,
}

impl<'a> FromContext<'a> {
    pub fn from_clausefrom(clause: &'a ClauseFrom, sqlos: &'a Sqlos) -> syn::Result<Self> {
        let alias_sqlos = clause.to_alias_sqlos(sqlos)?;
        Ok(Self { alias_sqlos })
    }
}
