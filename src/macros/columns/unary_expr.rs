use syn::Token;

use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub enum ColExprUnary {
    Minus(Box<ColExpr>),
    Not(Box<ColExpr>),
}

impl quote::ToTokens for ColExprUnary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Minus(p) => {
                "-".to_tokens(tokens);
                p.to_tokens(tokens)
            }
            Self::Not(p) => {
                "!".to_tokens(tokens);
                p.to_tokens(tokens)
            }
        }
    }
}

impl ColumnToSql for ColExprUnary {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        match self {
            Self::Minus(m) => {
                let mut qr = m.as_ref().column_to_sql(ctx)?;
                qr.prepend_str("-");
                Ok(qr)
            }
            Self::Not(n) => {
                let mut qr = n.as_ref().column_to_sql(ctx)?;
                qr.prepend_str(" NOT ");
                Ok(qr)
            }
        }
    }
}

impl ColExprUnary {
    pub fn get_next_unary(input: syn::parse::ParseStream) -> syn::Result<&str> {
        if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            Ok("!")
        } else if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            Ok("-")
        } else {
            Ok("")
        }
    }
}

pub fn unarize(colexp: ColExpr, unary: &str) -> ColExpr {
    match unary {
        "!" => ColExprUnary::Not(Box::new(colexp)).into(),
        "-" => ColExprUnary::Minus(Box::new(colexp)).into(),
        _ => colexp,
    }
}
