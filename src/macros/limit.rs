use syn::Token;

use super::kw;
use super::ColExpr;
use super::ColumnToSql;

#[derive(Debug)]
pub struct Limit {
    limit: ColExpr,
    offset: Option<ColExpr>,
}

impl syn::parse::Parse for Limit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::limit) {
            Limit::parse_limit(input)
        } else if input.peek(kw::page) {
            Limit::parse_page(input)
        } else {
            Err(input.error("expect `limit` or `page` keyword"))
        }
    }
}

// parsing limit and page
impl Limit {
    fn parse_limit(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::limit>()?;
        let reste;
        parse_possible_bracketed!(input, reste);
        let limit = reste.parse()?;
        let offset = if reste.peek(Token![,]) {
            reste.parse::<Token![,]>()?;
            Some(reste.parse()?)
        } else {
            None
        };
        Ok(Limit { limit, offset })
    }

    fn parse_page(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::page>()?;
        let reste;
        parse_possible_bracketed!(input, reste);
        let page_nb: ColExpr = reste.parse()?;
        reste.parse::<Token![,]>()?;
        let page_size: ColExpr = reste.parse()?;
        let offset: syn::Expr = syn::parse_quote! {(#page_nb - 1)*#page_size};
        Ok(Limit {
            limit: page_size,
            offset: Some(ColExpr::Value(offset)),
        })
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
