use crate::{
    error::SqloError,
    macros::{kw, ColExpr, ColumnToSql, Context, Fragment, Generator},
};

use syn::{punctuated::Punctuated, Token};

#[derive(Debug, Clone)]
pub struct OrderElem {
    column: ColExpr,
    sens: bool,
}

impl syn::parse::Parse for OrderElem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sens = if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            false
        } else {
            true
        };
        let column = input.parse::<ColExpr>()?;
        match column {
            ColExpr::Call(_) | ColExpr::Field(_) | ColExpr::Ident(_) => {
                Ok(OrderElem { column, sens })
            }
            _ => Err(syn::Error::new_spanned(
                column,
                "order_by only supports identifier, related identifier or function",
            )),
        }
    }
}

impl quote::ToTokens for OrderElem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let colexpr = &self.column;
        if self.sens {
            quote::quote! {order_by #colexpr}
        } else {
            quote::quote! { order_by -#colexpr }
        }
        .to_tokens(tokens);
    }
}

impl ColumnToSql for OrderElem {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, crate::error::SqloError> {
        let sens = if self.sens { "" } else { " DESC" };
        let mut res = self.column.column_to_sql(ctx)?;
        res.append_str(sens);
        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct OrderBy(Punctuated<OrderElem, Token![,]>);

impl syn::parse::Parse for OrderBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::order_by>()?;
        let reste;
        parse_possible_bracketed!(input, reste);
        Ok(OrderBy(Punctuated::parse_separated_nonempty(reste)?))
    }
}

impl ColumnToSql for OrderBy {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, crate::error::SqloError> {
        ctx.context.push(Context::OrderBy);
        let mut res = self.0.iter().fold(
            Ok(Fragment::default()),
            |acc: Result<Fragment, SqloError>, nex| Ok(acc.unwrap() + nex.column_to_sql(ctx)?), //unwrap ok can't be None
        )?;
        res.prepend_str(" ORDER BY ");
        ctx.context.pop();
        Ok(res)
    }
}
