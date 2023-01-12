macro_rules! impl_from_for_clause_variant {
    ($struct:ident $variant:ident $kw:ident) => {
        impl From<$struct> for crate::macros::common::Clause {
            fn from(c: $struct) -> Self {
                crate::macros::common::Clause::$variant(c)
            }
        }
    };
}

macro_rules! impl_validate_for_clause_variant {
    ($struct:ident) => {
        impl crate::macros::common::Validate for $struct {
            fn validate(&self, sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
                self.tokens.validate(sqlos)
            }
        }
    };
}
macro_rules! impl_parse_for_clause {
    ($struct:ident  $kw:ident) => {
        impl syn::parse::Parse for $struct {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                // parse the clause
                let keyword = input.parse::<crate::macros::common::SqlKeyword>()?;
                // }

                Ok($struct {
                    keyword,
                    tokens: input.parse()?,
                })
            }
        }
    };
}

macro_rules! impl_stry_for_clause {
    ($struct:ident $name:literal) => {
        #[cfg(test)]
        impl crate::macros::common::stringify::Stringify for $struct {
            fn stry(&self) -> String {
                format!("{} {}", $name, self.tokens.stry())
            }
        }
    };
}

macro_rules! impl_trait_to_tokens_for_clause {
    ($struct:ident, $($fields:ident),+) => {
        impl quote::ToTokens for $struct {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                self.keyword.to_tokens(tokens);
                $(self.$fields.to_tokens(tokens);)+
            }
        }
    };
}
