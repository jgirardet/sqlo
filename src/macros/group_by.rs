use syn::{punctuated::Punctuated, Token};

use crate::error::SqloError;

use super::{kw, ColExpr, ColumnToSql, SqlQuery};

pub struct GroupBy(Vec<ColExpr>);

impl syn::parse::Parse for GroupBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::group_by>()?;
        let reste;
        parse_possible_bracketed!(input, reste);
        Ok(GroupBy(
            Punctuated::<ColExpr, Token![,]>::parse_separated_nonempty(reste)?
                .into_iter()
                .collect(),
        ))
    }
}

impl ColumnToSql for GroupBy {
    fn column_to_sql(
        &self,
        ctx: &mut super::SqlResult,
    ) -> Result<super::SqlQuery, crate::error::SqloError> {
        let mut qr = self.0.iter().fold(
            Ok(SqlQuery::default()),
            |acc: Result<SqlQuery, SqloError>, nex| Ok(acc.unwrap() + nex.column_to_sql(ctx)?),
        )?;
        qr.prepend_str(" GROUP BY ");
        Ok(qr)
    }
}
