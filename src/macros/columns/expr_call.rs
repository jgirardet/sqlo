use std::collections::HashSet;

use darling::util::IdentString;
use itertools::Itertools;
use proc_macro2::{Delimiter, Group, TokenStream};
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

use crate::{
    error::SqloError,
    macros::{SqlQuery, SqlResult},
};

use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct ColExprCall {
    pub base: IdentString,
    pub args: Punctuated<ColExpr, Token![,]>,
}

impl quote::ToTokens for ColExprCall {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut token_group = TokenStream::new();
        self.base.to_tokens(&mut token_group);
        let mut g2 = Group::new(Delimiter::Parenthesis, self.args.to_token_stream());
        g2.set_span(self.args.span());
        g2.to_tokens(&mut token_group);
        token_group.to_tokens(tokens);
    }
}

impl ColumnToSql for ColExprCall {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        let mut args = vec![];
        let mut params = vec![];
        for arg in self.args.iter() {
            args.push(arg.column_to_sql(ctx)?);
        }
        let query = format!(
            "{}({})",
            &self.base,
            args.iter().map(|x| &x.query).join(" ,")
        );
        let mut joins = HashSet::new();
        for j in args {
            joins.extend(j.joins);
            params.extend(j.params);
        }
        Ok(SqlQuery {
            query,
            params,
            joins,
        })
    }
}
