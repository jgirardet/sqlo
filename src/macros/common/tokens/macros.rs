macro_rules! impl_trait_to_tokens_for_tokens {
    ($struct:ident, $($fields:ident),+) => {
        impl quote::ToTokens for $struct {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                $(self.$fields.to_tokens(tokens);)+
            }
        }
    };
}
macro_rules! impl_trait_to_tokens_for_sqltoken {
    ($($ident:ident),+) => {

        impl quote::ToTokens for SqlToken {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                match self {

                    $(Self::$ident(v)=>v.to_tokens(tokens),)+
                };
            }
        }

    };

}

macro_rules! impl_from_kw_for_sqltoken {
    ($($kw:ident $variant:ident),+) => {
        $(
        impl From<kw::$kw> for SqlToken {
            fn from(value: kw::$kw) -> Self {
                SqlToken::Keyword(SqlKeyword::$kw(value))
            }
        }
        )+
    };
}

macro_rules! impl_from_tokens_for_sqltoken {
    ($(($token:ident, $sqltoken:ident)),+) => {
        $(
            impl From<$token> for SqlToken {
                fn from(c: $token) -> Self {
                    SqlToken::$sqltoken(c)
                }
            }
        )+
    };
}
