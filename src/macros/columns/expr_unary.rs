use syn::Token;

use crate::macros::{ColumnToSql, Context};

use super::ColExpr;

#[derive(Debug, Clone)]
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
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        ctx.context.push(Context::Unary);
        let res = match self {
            Self::Minus(m) => {
                let mut qr = m.as_ref().column_to_sql(ctx)?;
                qr.prepend_str("-");
                qr
            }
            Self::Not(n) => {
                let mut qr = n.as_ref().column_to_sql(ctx)?;
                qr.prepend_str(" NOT ");
                qr
            }
        };
        ctx.context.pop();
        Ok(res)
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
