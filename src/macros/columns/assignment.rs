use syn::{punctuated::Punctuated, Token};

use crate::macros::{ColumnToSql, Fragment};

use super::ColExpr;

#[derive(Debug, Clone)]
pub struct Assign {
    pub lhs: ColExpr,
    pub rhs: ColExpr,
}

impl syn::parse::Parse for Assign {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        input.parse::<Token!(=)>()?;
        let rhs = input.parse()?;
        Ok(Self { lhs, rhs })
    }
}

impl ColumnToSql for &Assign {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        let mut lhs = self.lhs.column_to_sql(ctx)?;
        lhs.append_str(" =");
        Ok(lhs.add_no_comma(self.rhs.column_to_sql(ctx)?))
    }
}

#[derive(Debug, Clone)]
pub struct Assigns(Vec<Assign>);

impl syn::parse::Parse for Assigns {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(
            Punctuated::<Assign, Token!(,)>::parse_separated_nonempty(input)?
                .into_iter()
                .collect(),
        ))
    }
}

impl ColumnToSql for Assigns {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        Fragment::from_iterator(&self.0, ctx)
    }
}
