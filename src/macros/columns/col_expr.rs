use darling::util::IdentString;
use syn::{
    parenthesized, parse::Parse, punctuated::Punctuated, Expr, ExprCall, ExprField, ExprIndex,
    ExprPath, Lit, Token,
};

use crate::{
    error::SqloError,
    macros::{unarize, Operator, SqlQuery, SqlResult},
};

use super::{ColExprCall, ColExprField, ColExprOp, ColExprParen, ColExprUnary, ColumnToSql};

#[derive(Debug)]
pub enum ColExpr {
    Ident(IdentString),
    Call(ColExprCall),
    Field(ColExprField),
    Literal(Lit),
    Value(Expr),
    Operation(ColExprOp),
    Paren(ColExprParen),
    Unary(ColExprUnary),
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
            Self::Paren(p) => p.to_tokens(tokens),
            Self::Unary(p) => p.to_tokens(tokens),
        }
    }
}

impl syn::parse::Parse for ColExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let unary = ColExprUnary::get_next_unary(input)?;
        let initial_col = if input.peek(syn::token::Paren) {
            parse_paren(input)?
        } else {
            parse_initial(input)?
        };
        let unarized_col = unarize(initial_col, unary);
        if Operator::next_is_supported_op(&input) {
            parse_operation(input, unarized_col)
        } else {
            Ok(unarized_col)
        }
    }
}

fn parse_paren(input: syn::parse::ParseStream) -> syn::Result<ColExpr> {
    let content;
    parenthesized!(content in input);
    let seq: Punctuated<ColExpr, Token![,]> = Punctuated::parse_separated_nonempty(&content)?;
    Ok(ColExprParen::new(seq.into_iter().collect()).into())
}

fn parse_initial(input: syn::parse::ParseStream) -> syn::Result<ColExpr> {
    let res = if input.peek(syn::Ident) {
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
            let args: Punctuated<ColExpr, Token![,]> = content.parse_terminated(ColExpr::parse)?;
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
    Ok(res)
}

fn parse_operation(input: syn::parse::ParseStream, lhs: ColExpr) -> syn::Result<ColExpr> {
    let op = input.parse::<Operator>()?;
    let rhs = input.parse::<ColExpr>()?;
    Ok(ColExpr::Operation(ColExprOp {
        lhs: Box::new(lhs),
        op,
        rhs: Box::new(rhs),
    }))
}

impl ColumnToSql for ColExpr {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        match self {
            Self::Ident(ident) => ident.column_to_sql(ctx),
            Self::Call(col_expr_call) => col_expr_call.column_to_sql(ctx),
            Self::Field(col_expr_field) => col_expr_field.column_to_sql(ctx),
            Self::Literal(l) => l.column_to_sql(ctx),
            Self::Value(expr_value) => expr_value.column_to_sql(ctx),
            Self::Operation(expr_op) => expr_op.column_to_sql(ctx),
            Self::Asterisk => Ok("*".to_string().into()),
            Self::Paren(p) => p.column_to_sql(ctx),
            Self::Unary(p) => p.column_to_sql(ctx),
        }
    }
}

macro_rules! impl_from_variant_for_colexpr {
    ($($variant:ident $target:ident),+) => {
        $(
        impl From<$target> for ColExpr {
            fn from(c: $target) -> Self {
                ColExpr::$variant(c)
            }
        }
        )+
    };
}

impl_from_variant_for_colexpr!(
    Ident IdentString,
    Call ColExprCall,
    Field ColExprField,
    Literal Lit,
    Value Expr,
    Operation ColExprOp,
    Paren ColExprParen,
    Unary ColExprUnary
);

// we support only a fex expr variant and we want to avoid parsing syn cast expr
fn parse_supported_expr(input: &syn::parse::ParseStream) -> Result<Expr, syn::Error> {
    let mut fork = input.fork();
    if fork.parse::<ExprIndex>().is_ok() {
        return Ok(input.parse::<ExprIndex>()?.into());
    }
    fork = input.fork();
    if fork.parse::<ExprField>().is_ok() {
        return Ok(input.parse::<ExprField>()?.into());
    }
    fork = input.fork();
    if fork.parse::<ExprCall>().is_ok() {
        return Ok(input.parse::<ExprCall>()?.into());
    }
    fork = input.fork();
    if fork.parse::<ExprPath>().is_ok() {
        return Ok(input.parse::<ExprPath>()?.into());
    }
    Err(input.error("Expression not supported as argument"))
}
