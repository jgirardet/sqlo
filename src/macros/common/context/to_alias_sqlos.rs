use crate::{
    macros::common::{ClauseTrait, SqlToken},
    sqlos::Sqlos,
};

use super::{alias_sqlo::AliasSqlos, AliasSqlo};

pub trait ToAliasSqlos<'a, 'b>: ClauseTrait {
    fn to_alias_sqlos(&'b self, sqlos: &'a Sqlos) -> syn::Result<AliasSqlos<'a, 'b>> {
        let mut res = vec![];

        for t in self.sqltokens() {
            match t {
                SqlToken::Cast(token_cast) => {
                    if let SqlToken::Ident(ref ti) = *token_cast.initial {
                        let sqlo = sqlos.get(ti.as_str())?;
                        match token_cast.alias.as_ref() {
                            SqlToken::Ident(i) => res.push(AliasSqlo::Alias(sqlo, i)),
                            _ => return_error!(&token_cast.alias, "Should be identifier"),
                        }
                    }
                }
                SqlToken::Ident(ti) => {
                    let sqlo = sqlos.get(ti.as_str())?;
                    res.push(AliasSqlo::Ident(sqlo));
                }
                _ => unreachable!("From clause has only Cast and Ident variant"),
            }
        }
        Ok(res.into())
    }
}
