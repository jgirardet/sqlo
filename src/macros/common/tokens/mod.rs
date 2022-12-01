macro_rules! impl_to_tokens_for_tokens {
    ($struct:ident, $($fields:ident),+) => {
        impl quote::ToTokens for $struct {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                $(self.$fields.to_tokens(tokens);)+
            }
        }
    };
}

mod sql_tokens;
mod token_binary;
mod token_call;
mod token_cast;
mod token_field;
mod token_ident;
mod token_literal;
mod token_operator;
mod token_paren;
mod token_seq;

pub use sql_tokens::SqlToken;
pub use token_binary::TokenBinary;
pub use token_call::TokenCall;
pub use token_cast::{CastSeparator, TokenCast};
pub use token_field::TokenField;
pub use token_ident::TokenIdent;
pub use token_literal::TokenLit;
pub use token_operator::TokenOperator;
pub use token_paren::TokenParen;
pub use token_seq::TokenSeq;
