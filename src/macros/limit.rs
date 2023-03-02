use syn::bracketed;
use syn::Token;

use super::kw;
use super::ColExpr;
use super::ColumnToSql;

pub struct Limit {
    limit: ColExpr,
    offset: Option<ColExpr>,
}

impl syn::parse::Parse for Limit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::limit>()?;
        let content;
        let reste = if input.peek(syn::token::Bracket) {
            bracketed!(content in input);
            &content
        } else {
            input
        };
        let limit = reste.parse()?;
        let offset = if reste.peek(Token![,]) {
            reste.parse::<Token![,]>()?;
            Some(reste.parse()?)
        } else {
            None
        };
        Ok(Limit { limit, offset })
    }
}

impl ColumnToSql for Limit {
    fn column_to_sql(
        &self,
        ctx: &mut super::SqlResult,
    ) -> Result<super::SqlQuery, crate::error::SqloError> {
        let mut limit = self.limit.column_to_sql(ctx)?;
        limit.prepend_str(" LIMIT ");
        if let Some(offset) = &self.offset {
            let offset = offset.column_to_sql(ctx)?;
            limit.append_str(" OFFSET");
            Ok(limit.add_no_comma(offset))
        } else {
            Ok(limit)
        }
    }
}
