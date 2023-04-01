use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;

#[derive(Debug, Clone, Copy, Default)]
pub enum Fetch {
    #[default]
    One,
    All,
    Stream,
    Optional,
    None,
}

impl syn::parse::Parse for Fetch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(Self::All)
        } else if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            Ok(Self::One)
        } else if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
            Ok(Self::Stream)
        } else if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
            Ok(Self::Optional)
        } else {
            Ok(Self::None)
        }
    }
}

impl Fetch {
    pub fn is_returning(&self) -> bool {
        !matches!(self, Self::None)
    }
}

impl quote::ToTokens for Fetch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::All => quote![fetch_all].to_tokens(tokens),
            Self::Stream => quote![fetch].to_tokens(tokens),
            Self::Optional => quote![fetch_optional].to_tokens(tokens),
            Self::One => quote![fetch_one].to_tokens(tokens),
            Self::None => quote![execute].to_tokens(tokens),
        }
    }
}
