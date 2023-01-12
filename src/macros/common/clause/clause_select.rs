use crate::{
    macros::common::{kw, QueryContext, SqlKeyword, SqlToken, Sqlize, Sqlized, TokenSeq, Validate},
    sqlos::Sqlos,
};

use super::{clause_from::AliasSqlo, ClauseFrom};

#[derive(Debug)]
pub struct ClauseSelect {
    pub keyword: SqlKeyword,
    pub tokens: TokenSeq,
    pub distinct: Option<SqlKeyword>,
}

impl_from_for_clause_variant!(ClauseSelect Select SELECT);
impl_stry_for_clause!(ClauseSelect "SELECT");
impl_trait_to_tokens_for_clause!(ClauseSelect, distinct, tokens);

impl Validate for ClauseSelect {
    fn validate(&self, sqlos: &Sqlos) -> syn::Result<()> {
        // those variants should be inside TokenCast, not alone
        for t in &self.tokens {
            match t {
                SqlToken::ExprCall(_)
                | SqlToken::ExprBinary(_)
                | SqlToken::ExprParen(_)
                | SqlToken::Literal(_) => {
                    return_error!(t, "Must be followed by `AS` + `column name`.")
                }
                _ => {}
            };
        }
        self.tokens.validate(sqlos)
    }
}

impl syn::parse::Parse for ClauseSelect {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let keyword = if input.peek(crate::macros::common::kw::SELECT) {
            input.parse::<crate::macros::common::SqlKeyword>()?
        } else {
            kw::SELECT(input.span()).into()
        };

        let distinct = if input.peek(kw::DISTINCT) {
            let dis: SqlKeyword = input.parse::<kw::DISTINCT>()?.into();
            dis.into()
        } else {
            None
        };

        Ok(ClauseSelect {
            keyword,
            tokens: input.parse()?,
            distinct,
        })
    }
}

impl Sqlize for ClauseSelect {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        self.keyword.sselect(acc, context)?;
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
    pub query_context: QueryContext,
}

impl<'a> SelectContext<'a> {
    pub fn from_clausefrom(
        clause: &'a ClauseFrom,
        sqlos: &'a Sqlos,
        query_context: QueryContext,
    ) -> syn::Result<Self> {
        let alias_sqlos = clause.to_alias_sqlos(sqlos)?;
        Ok(Self {
            alias_sqlos: alias_sqlos,
            query_context,
        })
    }

    pub fn lower(&mut self) {
        self.query_context = self.query_context.lower()
    }
}
