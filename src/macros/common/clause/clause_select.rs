use crate::{
    macros::common::{kw, SqlKeyword, Sqlize, Sqlized, TokenSeq},
    sqlos::Sqlos,
};

use super::{clause_from::AliasSqlo, ClauseFrom};

#[derive(Debug)]
pub struct ClauseSelect {
    pub tokens: TokenSeq,
    pub distinct: Option<SqlKeyword>,
}

impl_from_validate_for_clause_variant!(ClauseSelect Select SELECT);
impl_stry_for_clause!(ClauseSelect "SELECT");
impl syn::parse::Parse for ClauseSelect {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(crate::macros::common::kw::SELECT) {
            input.parse::<crate::macros::common::kw::SELECT>()?;
        }

        let distinct = if input.peek(kw::DISTINCT) {
            let dis: SqlKeyword = input.parse::<kw::DISTINCT>()?.into();
            dis.into()
        } else {
            None
        };

        Ok(ClauseSelect {
            // context: Context::Select,
            tokens: input.parse()?,
            distinct,
        })
    }
}

impl Sqlize for ClauseSelect {
    fn sselect(&self, acc: &mut Sqlized, context: &SelectContext) -> syn::Result<()> {
        acc.append_sql(format!("SELECT"));
        if let Some(distinct) = &self.distinct {
            acc.append_sql(distinct.to_string());
        }
        self.tokens.sselect(acc, context)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SelectContext<'a> {
    pub alias_sqlos: Vec<AliasSqlo<'a>>,
}

impl<'a> SelectContext<'a> {
    pub fn from_clausefrom(clause: &'a ClauseFrom, sqlos: &'a Sqlos) -> syn::Result<Self> {
        let alias_sqlos = clause.to_alias_sqlos(sqlos)?;
        Ok(Self { alias_sqlos })
    }
}
