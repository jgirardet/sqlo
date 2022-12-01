use syn::Token;
use syn::{BinOp, Expr, ExprPath, Ident};

use crate::macros::common::keyword::{kw, peek_keyword, SqlKeyword};

use super::{
    TokenBinary, TokenCall, TokenCast, TokenField, TokenIdent, TokenLit, TokenOperator, TokenParen,
    TokenSeq,
};

#[derive(Debug)]
pub enum SqlToken {
    Ident(TokenIdent),
    Literal(TokenLit),
    Keyword(SqlKeyword),
    Operator(TokenOperator),
    Cast(TokenCast),
    ExprField(TokenField),
    ExprBinary(TokenBinary),
    ExprParen(TokenParen),
    ExprCall(TokenCall),
    ExprSeq(TokenSeq),
}

impl syn::parse::Parse for SqlToken {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let start = input.parse::<Expr>()?.try_into()?;
        let cast_sep = input.parse()?; // avant la suite,  pour pouvoir utiliser is_empty
        if input.peek(Token![,]) || peek_keyword(input) || input.is_empty() {
            Ok(start)
        } else {
            let alias = input.parse()?;
            Ok(SqlToken::Cast(TokenCast::new(start, alias, cast_sep)))
        }
    }
}

macro_rules! impl_to_tokens_for_sqltoken {
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

impl_to_tokens_for_sqltoken!(
    Ident, Literal, Keyword, Operator, Cast, ExprBinary, ExprCall, ExprField, ExprSeq, ExprParen
);

impl From<Ident> for SqlToken {
    fn from(ident: Ident) -> Self {
        SqlToken::Ident(ident.into())
    }
}

impl TryFrom<Expr> for SqlToken {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        Ok(match expr {
            Expr::Path(ref p) => SqlToken::Ident(p.try_into()?),
            Expr::Lit(l) => SqlToken::Literal(l.try_into()?),
            Expr::Field(_) => SqlToken::ExprField(expr.try_into()?),
            Expr::Call(_) => SqlToken::ExprCall(expr.try_into()?),
            Expr::Paren(_) => SqlToken::ExprParen(expr.try_into()?),
            Expr::Binary(_) => SqlToken::ExprBinary(expr.try_into()?),
            _ => return_error!(expr, "Not a valid expression"),
        })
    }
}

impl TryFrom<&ExprPath> for SqlToken {
    type Error = syn::Error;
    fn try_from(p: &ExprPath) -> Result<Self, Self::Error> {
        Ok(SqlToken::Ident(p.try_into()?))
    }
}

macro_rules! impl_into_kw {
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

impl_into_kw!(FROM From, WHERE Where, AS As, DISTINCT Distinct, JOIN Join, SELECT Select);

impl TryFrom<BinOp> for SqlToken {
    type Error = syn::Error;
    fn try_from(op: BinOp) -> syn::Result<Self> {
        Ok(SqlToken::Operator(op.try_into()?))
    }
}

macro_rules! impl_from_tokens_to_sqltoken {
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

impl_from_tokens_to_sqltoken!(
    (TokenIdent, Ident),
    (TokenLit, Literal),
    (TokenOperator, Operator),
    (TokenField, ExprField),
    (TokenCast, Cast),
    (TokenCall, ExprCall),
    (TokenBinary, ExprBinary),
    (TokenParen, ExprParen),
    (TokenSeq, ExprSeq)
);
