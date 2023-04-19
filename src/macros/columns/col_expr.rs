use darling::util::IdentString;
use syn::{
    parse::Parse, punctuated::Punctuated, Expr, ExprCall, ExprField, ExprIndex, ExprPath, Lit,
    Token,
};

use crate::{
    error::SqloError,
    macros::{unarize, ColumnToSql, Fragment, Generator, Operator},
    relations::Join,
};

use super::{
    ColExprCall, ColExprCase, ColExprField, ColExprOp, ColExprParen, ColExprSubSelect, ColExprUnary,
};

#[derive(Debug, Clone)]
pub enum ColExpr {
    Ident(IdentString),
    Call(ColExprCall),
    Case(ColExprCase),
    Field(ColExprField),
    Literal(Lit),
    Value(Expr),
    Operation(ColExprOp),
    Paren(ColExprParen),
    SubSelect(ColExprSubSelect),
    Unary(ColExprUnary),
    Asterisk,
}

impl quote::ToTokens for ColExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(tokens),
            Self::Field(f) => f.to_tokens(tokens),
            Self::Case(c) => c.to_tokens(tokens),
            Self::Call(c) => c.to_tokens(tokens),
            Self::Literal(l) => l.to_tokens(tokens),
            Self::Value(e) => e.to_tokens(tokens),
            Self::Operation(o) => o.to_tokens(tokens),
            Self::Asterisk => "*".to_tokens(tokens),
            Self::Paren(p) => p.to_tokens(tokens),
            Self::SubSelect(s) => s.to_tokens(tokens),
            Self::Unary(p) => p.to_tokens(tokens),
        }
    }
}

impl syn::parse::Parse for ColExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let unary = ColExprUnary::get_next_unary(input)?;
        let initial_col = if input.peek(syn::token::Paren) {
            input.parse::<ColExprParen>()?.into()
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

fn parse_initial(input: syn::parse::ParseStream) -> syn::Result<ColExpr> {
    let res = if input.peek(syn::Ident) {
        // let start to see if it starts with an Ident
        let fork = input.fork();
        if fork.peek(Token![.]) {
            //parse field : ident.field
            let ident = input.parse::<syn::Ident>()?;
            input.parse::<Token![.]>()?;
            let member = input.parse::<syn::Ident>()?;
            ColExprField::new(ident, member, Join::Inner).into()
        } else if fork.peek(Token![=]) && fork.peek2(Token![.]) {
            // parse left join ident=.field
            let ident = input.parse::<syn::Ident>()?;
            input.parse::<Token![=]>()?;
            input.parse::<Token![.]>()?;
            let member = input.parse::<syn::Ident>()?;
            ColExprField::new(ident, member, Join::Left).into()
        } else if fork.peek(syn::token::Paren) {
            // parse call: ident(...)
            ColExprCall {
                base: input.parse::<syn::Ident>()?.into(),
                args: input.parse::<ColExprParen>()?,
            }
            .into()
        } else if fork.peek(syn::token::Brace) {
            // parse subquery: exists {...}
            let ident = input.parse::<syn::Ident>()?;
            ColExprSubSelect::parse_with_ident(ident.into(), input)?.into()
        } else if fork.peek(syn::token::Bracket) {
            ColExpr::Value(input.parse::<ExprIndex>()?.into())
        } else {
            // nothing more so its a simple identifier
            ColExpr::Ident(input.parse::<syn::Ident>()?.into())
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
    } else if input.peek(syn::token::Brace) {
        ColExprSubSelect::parse_without_ident(input)?.into()
    } else if input.peek(Token![match]) {
        ColExprCase::parse(input)?.into()
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
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError> {
        match self {
            Self::Ident(ident) => ident.column_to_sql(ctx),
            Self::Call(col_expr_call) => col_expr_call.column_to_sql(ctx),
            Self::Case(c) => c.column_to_sql(ctx),
            Self::Field(col_expr_field) => col_expr_field.column_to_sql(ctx),
            Self::Literal(l) => l.column_to_sql(ctx),
            Self::Value(expr_value) => expr_value.column_to_sql(ctx),
            Self::Operation(expr_op) => expr_op.column_to_sql(ctx),
            Self::Asterisk => Ok("*".to_string().into()),
            Self::Paren(p) => p.column_to_sql(ctx),
            Self::SubSelect(p) => p.column_to_sql(ctx),
            Self::Unary(p) => p.column_to_sql(ctx),
        }
    }
}

impl ColumnToSql for &ColExpr {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError> {
        match *self {
            ColExpr::Ident(ident) => ident.column_to_sql(ctx),
            ColExpr::Call(col_expr_call) => col_expr_call.column_to_sql(ctx),
            ColExpr::Case(c) => c.column_to_sql(ctx),
            ColExpr::Field(col_expr_field) => col_expr_field.column_to_sql(ctx),
            ColExpr::Literal(l) => l.column_to_sql(ctx),
            ColExpr::Value(expr_value) => expr_value.column_to_sql(ctx),
            ColExpr::Operation(expr_op) => expr_op.column_to_sql(ctx),
            ColExpr::Asterisk => Ok("*".to_string().into()),
            ColExpr::Paren(p) => p.column_to_sql(ctx),
            ColExpr::SubSelect(p) => p.column_to_sql(ctx),
            ColExpr::Unary(p) => p.column_to_sql(ctx),
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
    SubSelect ColExprSubSelect,
    Unary ColExprUnary,
    Case ColExprCase

);

impl From<Punctuated<ColExpr, Token![,]>> for ColExpr {
    fn from(p: Punctuated<ColExpr, Token![,]>) -> Self {
        let cp: ColExprParen = p.into();
        cp.into()
    }
}

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
