use syn::{BinOp, Expr, ExprBinary};

use crate::utils::op_to_str;

use super::tok::Toks;
use super::totok::ToTok;

#[derive(Debug, Clone)]
pub enum WhereTokenizer {
    Mono(Box<syn::Expr>),
    Binary(syn::ExprBinary),
}

impl syn::parse::Parse for WhereTokenizer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let forked = input.fork();
        match input.parse::<ExprBinary>() {
            Ok(binary) => Ok(WhereTokenizer::Binary(binary)),
            Err(_) => match forked.parse::<syn::Expr>() {
                Ok(expr) => Ok(WhereTokenizer::Mono(Box::new(expr))),
                Err(e) => Err(syn::Error::new(e.span(), "Not a valid where expression")),
            },
        }
    }
}

pub(crate) fn parse_binary_eq(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) {
    // check for Null pattern
    if let Expr::Path(p) = right {
        if let Some(ident) = p.path.get_ident() {
            if ident == "None" {
                match left {
                    Expr::Field(_) | Expr::Path(_) => match op {
                        BinOp::Eq(_) | BinOp::Ne(_) => {
                            acc.null(left, op);
                            return;
                        }

                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            }
        }
    }
    // regular
    left.to_tok(acc);
    parse_operator(op, acc);
    right.to_tok(acc);
}

pub(crate) fn parse_binary_comp(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) {
    left.to_tok(acc);
    parse_operator(op, acc);
    right.to_tok(acc);
}

pub(crate) fn parse_binary_bool(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) {
    parse_binary_bool_member(left, acc, "before");
    parse_operator(op, acc);
    parse_binary_bool_member(right, acc, "after");
}

pub(crate) fn parse_binary_bool_member(expr: &Expr, acc: &mut Toks, err_msg: &str) {
    match expr {
        Expr::Binary(b) => b.to_tok(acc),
        Expr::Unary(u) => u.to_tok(acc),
        Expr::Paren(p) => p.to_tok(acc),
        _ => acc.error(expr, &format!("Expression not supported {err_msg} and/or")),
    }
}

pub(crate) fn parse_operator(op: &BinOp, acc: &mut Toks) {
    acc.sign(op_to_str(op))
}
