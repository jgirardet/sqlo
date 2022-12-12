use syn::punctuated::Punctuated;
use syn::Token;
use syn::{BinOp, Expr, ExprPath, Ident};

use crate::macros::common::keyword::{kw, peek_keyword, SqlKeyword};
use crate::macros::common::{FromContext, SelectContext, Sqlize, Sqlized, Validate};

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

// ------------------ Various From/TryFrom conversion ------------------------- //

impl From<Ident> for SqlToken {
    fn from(ident: Ident) -> Self {
        SqlToken::Ident(ident.into())
    }
}

impl TryFrom<Expr> for SqlToken {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        Ok(match expr {
            Expr::Path(ref p) => p.try_into()?,
            Expr::Lit(l) => SqlToken::Literal(l.try_into()?),
            Expr::Field(_) => SqlToken::ExprField(expr.try_into()?),
            Expr::Call(_) => SqlToken::ExprCall(expr.try_into()?),
            Expr::Paren(_) | Expr::Tuple(_) => SqlToken::ExprParen(expr.try_into()?),
            Expr::Binary(_) => SqlToken::ExprBinary(expr.try_into()?),
            _ => return_error!(expr, "Not a valid expression"),
        })
    }
}

impl TryFrom<&ExprPath> for SqlToken {
    type Error = syn::Error;
    fn try_from(p: &ExprPath) -> Result<Self, Self::Error> {
        if let Some(ident) = p.path.get_ident() {
            if ident == "DISTINCT" {
                Ok(kw::DISTINCT(ident.span()).into())
            } else {
                Ok(SqlToken::Ident(p.try_into()?))
            }
        } else {
            return_error!(p, "Invalid expresion: `::` not supported")
        }
    }
}

impl TryFrom<Punctuated<Expr, Token![,]>> for SqlToken {
    type Error = syn::Error;

    fn try_from(punctuated: Punctuated<Expr, Token![,]>) -> Result<Self, Self::Error> {
        Ok(SqlToken::ExprSeq(punctuated.try_into()?))
    }
}

impl_from_kw_for_sqltoken!(FROM From, WHERE Where, AS As, DISTINCT Distinct, JOIN Join, SELECT Select);

impl TryFrom<BinOp> for SqlToken {
    type Error = syn::Error;
    fn try_from(op: BinOp) -> syn::Result<Self> {
        Ok(SqlToken::Operator(op.try_into()?))
    }
}

impl_from_tokens_for_sqltoken!(
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

// ------------------ Various Trait implementation ------------------------- //

impl syn::parse::Parse for SqlToken {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let start = input.parse::<Expr>()?.try_into()?;
        if let SqlToken::Keyword(_) = start {
            return Ok(start);
        }
        let cast_sep = input.parse()?; // avant la suite,  pour pouvoir utiliser is_empty
        if input.peek(Token![,]) || peek_keyword(input) || input.is_empty() {
            Ok(start)
        } else {
            let alias = input.parse()?;
            Ok(SqlToken::Cast(TokenCast::new(start, alias, cast_sep)))
        }
    }
}

impl_trait_to_tokens_for_sqltoken!(
    Ident, Literal, Keyword, Operator, Cast, ExprBinary, ExprCall, ExprField, ExprSeq, ExprParen
);

impl Validate for SqlToken {
    fn validate(&self, sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
        match self {
            Self::Ident(x) => x.validate(sqlos),
            Self::Cast(x) => x.validate(sqlos),
            Self::ExprBinary(x) => x.validate(sqlos),
            Self::ExprCall(x) => x.validate(sqlos),
            Self::ExprField(x) => x.validate(sqlos),
            Self::ExprParen(x) => x.validate(sqlos),
            Self::ExprSeq(x) => x.validate(sqlos),
            Self::Literal(x) => x.validate(sqlos),
            Self::Keyword(x) => x.validate(sqlos),
            Self::Operator(x) => x.validate(sqlos),
        }
    }
}

impl Sqlize for SqlToken {
    fn sselect(&self, acc: &mut Sqlized, context: &SelectContext) -> syn::Result<()> {
        match self {
            Self::Ident(x) => x.sselect(acc, context),
            Self::Cast(x) => x.sselect(acc, context),
            Self::Literal(x) => x.sselect(acc, context),
            Self::ExprField(x) => x.sselect(acc, context),
            Self::ExprParen(x) => x.sselect(acc, context),
            Self::ExprCall(x) => x.sselect(acc, context),
            Self::Operator(x) => x.sselect(acc, context),
            Self::ExprBinary(x) => x.sselect(acc, context),
            Self::ExprSeq(x) => x.sselect(acc, context),
            Self::Keyword(x) => x.sselect(acc, context),
        }
    }

    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        match self {
            Self::Ident(x) => x.ffrom(acc, context),
            Self::Cast(x) => x.ffrom(acc, context),
            // Self::ExprBinary(x) => x.select(acc, used_sqlos),
            // Self::ExprCall(x) => x.select(acc, used_sqlos),
            // Self::ExprField(x) => x.select(acc, used_sqlos),
            // Self::ExprParen(x) => x.select(acc, used_sqlos),
            // Self::ExprSeq(x) => x.select(acc, used_sqlos),
            // Self::Literal(x) => x.select(acc, used_sqlos),
            // Self::Keyword(x) => x.select(acc, used_sqlos),
            // Self::Operator(x) => x.select(acc, used_sqlos),
            _ => unimplemented!("not yet"),
        }
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for SqlToken {
    fn stry(&self) -> String {
        match self {
            Self::Ident(x) => x.stry(),
            Self::Cast(x) => x.stry(),
            Self::ExprBinary(x) => x.stry(),
            Self::ExprCall(x) => x.stry(),
            Self::ExprField(x) => x.stry(),
            Self::ExprParen(x) => x.stry(),
            Self::ExprSeq(x) => x.stry(),
            Self::Literal(x) => x.stry(),
            Self::Keyword(x) => x.stry(),
            Self::Operator(x) => x.stry(),
        }
    }
}
