use darling::util::IdentString;
use proc_macro2::TokenStream;
use syn::Token;

use crate::sqlos::Sqlos;

use super::{Phrase, Validate};

pub struct SqloAs {
    pub target_struct: IdentString,
    pub phrase: Phrase,
}

impl syn::parse::Parse for SqloAs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target_struct = input.parse::<syn::Ident>()?.into();
        input.parse::<Token![,]>()?;
        let phrase = input.parse()?;
        Ok(Self {
            target_struct,
            phrase,
        })
    }
}

impl SqloAs {
    pub fn expand(self, sqlos: &Sqlos) -> syn::Result<TokenStream> {
        self.phrase.validate(sqlos)?;
        let sqlized = self.phrase.sqlize(sqlos)?;
        let sql = sqlized.to_string();
        let params = sqlized.params();
        let target_struct = self.target_struct;
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&sql);
        }
        Ok(quote::quote! {
            sqlx::query_as![#target_struct, #sql,#(#params),*]
        })
    }
}
