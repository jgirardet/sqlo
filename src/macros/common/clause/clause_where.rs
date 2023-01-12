use crate::macros::common::{SqlKeyword, TokenSeq};

#[derive(Debug)]
pub struct ClauseWhere {
    keyword: SqlKeyword,
    tokens: TokenSeq,
}

impl_from_for_clause_variant!(ClauseWhere Where WHERE);
impl_validate_for_clause_variant!(ClauseWhere);
impl_parse_for_clause!(ClauseWhere  WHERE);
impl_stry_for_clause!(ClauseWhere "WHERE");
impl_trait_to_tokens_for_clause!(ClauseWhere, tokens);
