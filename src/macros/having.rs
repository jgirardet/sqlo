use super::{kw, ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct Having(ColExpr);

impl syn::parse::Parse for Having {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::having>()?;
        Ok(Having(input.parse()?))
    }
}

impl ColumnToSql for Having {
    fn column_to_sql(
        &self,
        ctx: &mut super::SqlResult,
    ) -> Result<super::SqlQuery, crate::error::SqloError> {
        let mut res = self.0.column_to_sql(ctx)?;
        res.prepend_str(" HAVING ");
        Ok(res)
    }
}
