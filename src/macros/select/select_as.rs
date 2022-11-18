use proc_macro2::TokenStream;
use syn::{Ident, Token};

use super::Select;

#[derive(Debug)]
pub struct SelectAs {
    query_as: Ident,
    select: Select,
}

impl syn::parse::Parse for SelectAs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let query_as = input.parse()?;
        input.parse::<Token![,]>()?;

        let select = input.parse::<Select>()?;

        let res = SelectAs { query_as, select };
        res.validate()
    }
}

impl SelectAs {
    pub fn expand(self) -> syn::Result<TokenStream> {
        let target_struct = self.query_as;
        let sql = self.select.to_sql()?;

        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&sql);
        }

        let res = quote::quote! {
            sqlx::query_as!(#target_struct, #sql)
        };
        Ok(res)
    }

    fn validate(self) -> syn::Result<Self> {
        Ok(self)
    }
}
