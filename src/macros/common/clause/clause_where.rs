use crate::macros::common::TokenSeq;

#[derive(Debug)]
pub struct ClauseWhere {
    tokens: TokenSeq,
}

impl_from_for_clause_variant!(ClauseWhere Where WHERE);
impl_validate_for_clause_variant!(ClauseWhere);
impl_parse_for_clause!(ClauseWhere  WHERE);
impl_stry_for_clause!(ClauseWhere "WHERE");
