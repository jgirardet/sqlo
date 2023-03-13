use crate::macros::{ColExpr, ColumnToSql, Context, Fragment, Generator};

#[derive(Debug)]
pub struct Where(ColExpr);

impl syn::parse::Parse for Where {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Where(input.parse()?))
    }
}

impl ColumnToSql for Where {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, crate::error::SqloError> {
        ctx.context.push(Context::Where);
        let mut res = self.0.column_to_sql(ctx)?;
        res.prepend_str(" WHERE ");
        ctx.context.pop();
        Ok(res)
    }
}
