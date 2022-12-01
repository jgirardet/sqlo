use syn::{Expr, ExprPath, Ident};

#[derive(Debug, Clone)]
pub struct TokenIdent {
    ident: Ident,
}

impl From<Ident> for TokenIdent {
    fn from(ident: Ident) -> Self {
        TokenIdent { ident }
    }
}

impl_to_tokens_for_tokens!(TokenIdent, ident);

impl syn::parse::Parse for TokenIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        Ok(TokenIdent { ident })
    }
}

impl From<&Ident> for TokenIdent {
    fn from(ident: &Ident) -> Self {
        TokenIdent {
            ident: ident.clone(),
            // span: ident.span(),
        }
    }
}

impl TryFrom<&ExprPath> for TokenIdent {
    type Error = syn::Error;
    fn try_from(p: &ExprPath) -> Result<Self, Self::Error> {
        if let Some(ident) = p.path.get_ident() {
            Ok(ident.into())
        } else {
            return_error!(p, "invalid column identifier")
        }
    }
}

impl TryFrom<Expr> for TokenIdent {
    type Error = syn::Error;
    fn try_from(p: Expr) -> Result<Self, Self::Error> {
        if let Expr::Path(ref expr_path) = p {
            return expr_path.try_into();
        }
        return_error!(p, "invalid column identifier")
    }
}
