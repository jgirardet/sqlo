use darling::util::IdentString;
use syn::{
    parenthesized, punctuated::Punctuated, BinOp, Expr, ExprCall, ExprField, ExprIndex, ExprPath,
    Lit, Token,
};

use crate::{error::SqloError, macros::SqlQuery, sqlo::Sqlo, sqlos::Sqlos};

use super::{expr_op::next_is_supported_op, ColExprCall, ColExprField, ColExprOp, ColumnToSql};

#[derive(Debug)]
pub enum ColExpr {
    Ident(IdentString),
    Call(ColExprCall),
    Field(ColExprField),
    Literal(Lit),
    Value(Expr),
    Operation(ColExprOp),
    Asterisk,
}

impl quote::ToTokens for ColExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(tokens),
            Self::Field(f) => f.to_tokens(tokens),
            Self::Call(c) => c.to_tokens(tokens),
            Self::Literal(l) => l.to_tokens(tokens),
            Self::Value(e) => e.to_tokens(tokens),
            Self::Operation(o) => o.to_tokens(tokens),
            Self::Asterisk => "*".to_tokens(tokens),
        }
    }
}

impl syn::parse::Parse for ColExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let col = if input.peek(syn::Ident) {
            // let start to see if it starts with an Ident
            let ident: syn::Ident = input.parse()?;
            if input.peek(Token![.]) {
                //parse field
                input.parse::<Token![.]>()?;
                let member = input.parse::<syn::Ident>()?;
                ColExpr::Field((ident, member).into())
            } else if input.peek(syn::token::Paren) {
                // parse call
                let content;
                parenthesized!(content in input);
                let args: Punctuated<ColExpr, Token![,]> =
                    content.parse_terminated(ColExpr::parse)?;
                ColExpr::Call(ColExprCall {
                    base: ident.into(),
                    args,
                })
            } else {
                // nothing more so its a simple identifier
                ColExpr::Ident(ident.into())
            }
        // it wasn't  an Ident, so is it something else ?
        } else if input.peek(Lit) {
            // parse literal arg
            ColExpr::Literal(input.parse::<Lit>()?)
        } else if input.peek(Token![::]) {
            // parse any other arg if supported
            input.parse::<Token![::]>()?;
            ColExpr::Value(parse_supported_expr(&input)?)
        } else if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            ColExpr::Asterisk
        } else {
            return Err(input.error("Sqlo: Invalid input"));
        };
        if next_is_supported_op(&input) {
            let sign = input.parse::<BinOp>()?;
            let rhs = input.parse::<ColExpr>()?;
            Ok(ColExpr::Operation(ColExprOp {
                lhs: Box::new(col),
                sign,
                rhs: Box::new(rhs),
            }))
        } else {
            Ok(col)
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
            Self::Operation(expr_op) => expr_op.column_to_sql(main_sqlo, sqlos),
            Self::Asterisk => Ok("*".to_string().into()),
        }
    }
}

// we support only a fex expr variant and we want to avoid parsing syn cast expr
fn parse_supported_expr(input: &syn::parse::ParseStream) -> Result<Expr, syn::Error> {
    let mut fork = input.fork();
    if let Ok(_) = fork.parse::<ExprIndex>() {
        return Ok(input.parse::<ExprIndex>()?.into());
    }
    fork = input.fork();
    if let Ok(_) = fork.parse::<ExprField>() {
        return Ok(input.parse::<ExprField>()?.into());
    }
    fork = input.fork();
    if let Ok(_) = fork.parse::<ExprCall>() {
        return Ok(input.parse::<ExprCall>()?.into());
    }
    fork = input.fork();
    if let Ok(_) = fork.parse::<ExprPath>() {
        return Ok(input.parse::<ExprPath>()?.into());
    }
    Err(input.error("Expression not supported as argument"))
}
