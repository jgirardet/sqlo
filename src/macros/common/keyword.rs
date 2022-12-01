pub mod kw {

    syn::custom_keyword!(AS);
    syn::custom_keyword!(DISTINCT);
    syn::custom_keyword!(FROM);
    syn::custom_keyword!(JOIN);
    syn::custom_keyword!(SELECT);
    syn::custom_keyword!(WHERE);
}

macro_rules! impl_sql_keyword {
    ($($name:ident),+) => {
        #[derive(Debug)]
        pub enum SqlKeyword {
        $($name(kw::$name),)+
        }

        impl quote::ToTokens for SqlKeyword {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                match *self {
                    $(Self::$name(key)=>key.to_tokens(tokens),)+
                }
            }
        }

        impl syn::parse::Parse for SqlKeyword {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                $(
                if input.peek(kw::$name) {
                    return Ok(SqlKeyword::$name(input.parse::<kw::$name>()?))
                }
            )+
                Err(input.error("Not a SqlKeyword"))
            }
        }

        pub fn peek_keyword(input: syn::parse::ParseStream) -> bool {
            $(input.peek(kw::$name))||+
        }
    };
}

impl_sql_keyword!(AS, DISTINCT, FROM, WHERE, SELECT, JOIN);
