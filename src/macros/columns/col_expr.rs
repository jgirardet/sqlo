use darling::util::IdentString;
use syn::{parenthesized, punctuated::Punctuated, Expr, Lit, Token};

use crate::{error::SqloError, macros::SqlQuery, sqlo::Sqlo, sqlos::Sqlos};

use super::{ColExprCall, ColExprField, ColumnToSql};

#[derive(Debug)]
pub enum ColExpr {
    Ident(IdentString),
    Call(ColExprCall),
    Field(ColExprField),
    Literal(Lit),
    Value(Expr),
}

impl quote::ToTokens for ColExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(tokens),
            Self::Field(f) => f.to_tokens(tokens),
            Self::Call(c) => c.to_tokens(tokens),
            Self::Literal(l) => l.to_tokens(tokens),
            Self::Value(e) => e.to_tokens(tokens),
        }
    }
}

impl syn::parse::Parse for ColExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if input.peek(Token![.]) {
                input.parse::<Token![.]>()?;
                let member = input.parse::<syn::Ident>()?;
                Ok(ColExpr::Field((ident, member).into()))
            } else if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                let args: Punctuated<ColExpr, Token![,]> =
                    content.parse_terminated(ColExpr::parse)?;
                Ok(ColExpr::Call(ColExprCall {
                    base: ident.into(),
                    args,
                }))
            } else {
                Ok(ColExpr::Ident(ident.into()))
            }
        } else if input.peek(Lit) {
            Ok(ColExpr::Literal(input.parse::<Lit>()?))
        } else if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            Ok(ColExpr::Value(input.parse::<Expr>()?))
        } else {
            Err(input.error("Invalid input"))
        }
    }
}

impl ColumnToSql for ColExpr {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        match self {
            Self::Ident(ident) => Ok(main_sqlo.column(ident.as_ident())?.into()),
            Self::Call(col_expr_call) => col_expr_call.column_to_sql(main_sqlo, sqlos),
            Self::Field(col_expr_field) => col_expr_field.column_to_sql(main_sqlo, sqlos),
            Self::Literal(l) => l.column_to_sql(main_sqlo, sqlos),
            Self::Value(expr_value) => expr_value.column_to_sql(main_sqlo, sqlos),
        }
    }
}
