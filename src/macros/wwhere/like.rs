use darling::util::IdentString;
use syn::{ExprField, LitStr, Token};

#[derive(Debug, Clone)]
pub struct Like {
    pub field: LikeField,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum LikeField {
    Direct(IdentString),
    Related(ExprField),
}

impl syn::parse::Parse for Like {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field = match input.fork().parse::<ExprField>() {
            Ok(_) => LikeField::Related(input.parse::<ExprField>()?),
            Err(_) => match input.parse::<syn::Ident>() {
                Ok(ident) => LikeField::Direct(ident.into()),
                Err(_) => return Err(input.error("must be field identifier or related identifier")),
            },
        };
        input.parse::<Token!(,)>()?;
        let text = input.parse::<LitStr>()?.value();
        Ok(Like { field, text })
    }
}
