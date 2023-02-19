use std::fmt::Display;

use darling::util::IdentString;
use syn::{Ident, LitStr, Token};

#[derive(Debug, Clone)]
pub struct Like {
    field: IdentString,
    text: String,
}

impl syn::parse::Parse for Like {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field = input.parse::<Ident>()?;
        input.parse::<Token!(,)>()?;
        let text = input.parse::<LitStr>()?;
        Ok(Like {
            field: field.into(),
            text: text.value(),
        })
    }
}

impl Display for Like {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} LIKE '{}'", &self.field, &self.text)
    }
}
