use std::fmt::Display;

use darling::util::IdentString;

use quote::ToTokens;

use super::{
    tokenizer::{parse_operator, WhereTokenizer},
    totok::ToTok,
    Like,
};

#[derive(Debug, Clone, Default)]
pub struct Toks(Vec<Tok>);

impl From<&WhereTokenizer> for Toks {
    fn from(b: &WhereTokenizer) -> Self {
        let mut t = Self::default();
        match b {
            WhereTokenizer::Binary(b) => b.to_tok(&mut t),
            // we don't use Expr.to_tok, to keep separated the behavior at first node.
            WhereTokenizer::Mono(m) => match m.as_ref() {
                // if only parenthesis, its first parsed as a group
                syn::Expr::Group(syn::ExprGroup { expr, .. }) => match **expr {
                    syn::Expr::Paren(ref p) => p.to_tok(&mut t),
                    syn::Expr::Range(ref p) => p.to_tok(&mut t),
                    _ => t.error(m, "Only Binary, Parenthesis and Not expression supported"),
                },
                syn::Expr::Range(ref p) => p.to_tok(&mut t),
                syn::Expr::Paren(ref p) => p.to_tok(&mut t),
                syn::Expr::Unary(ref p) => p.to_tok(&mut t),
                syn::Expr::Macro(ref p) => p.to_tok(&mut t),

                _ => t.error(m, "Only Binary, Parenthesis and  Not expression supported"),
            },
        };
        t
    }
}

impl Toks {
    pub fn field(&mut self, ident: &syn::Ident) {
        self.0.push(Tok::Field(ident.clone().into()))
    }

    pub fn foreign_key(&mut self, field: &syn::ExprField) {
        self.0.push(Tok::ForeignKey(field.clone()))
    }
    pub fn iin(&mut self, toks: &Toks) {
        self.0.push(Tok::In(toks.clone()))
    }

    pub fn like(&mut self, like: Like) {
        self.0.push(Tok::Like(like))
    }

    pub fn not(&mut self, toks: &Toks) {
        self.0.push(Tok::Not(toks.clone()))
    }

    pub fn null(&mut self, expr: &syn::Expr, op: &syn::BinOp) {
        let mut toks = Toks::default();
        expr.to_tok(&mut toks);
        match op {
            syn::BinOp::Eq(_) | syn::BinOp::Ne(_) => parse_operator(op, &mut toks),
            _ => self.error(op, "Operator not supported with None/Null expression"),
        }
        self.0.push(Tok::Null(toks));
    }

    pub fn paren(&mut self, toks: &Toks) {
        self.0.push(Tok::Paren(toks.clone()))
    }

    pub fn sign(&mut self, si: &str) {
        self.0.push(Tok::Sign(si.to_string()))
    }

    pub fn value(&mut self, val: &syn::Expr) {
        self.0.push(Tok::Value(val.clone()))
    }

    pub fn error<T: ToTokens, U: Display>(&mut self, token: T, msg: U) {
        self.0.push(Tok::Error(syn::Error::new_spanned(token, msg)));
    }
}

impl IntoIterator for Toks {
    type Item = Tok;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone)]
pub enum Tok {
    // Call(syn::Expr),
    Field(IdentString),
    ForeignKey(syn::ExprField),
    In(Toks),
    Like(Like),
    Null(Toks),
    Not(Toks),
    Paren(Toks),
    Sign(String),
    Value(syn::Expr),
    Error(syn::Error),
}
