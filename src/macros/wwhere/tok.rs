use std::fmt::Display;

use itertools::Itertools;
use quote::ToTokens;

use crate::utils::display_expr;

use super::{
    tokenizer::{parse_operator, WhereTokenizer},
    totok::ToTok,
};

#[derive(Debug, Clone, Default)]
pub struct Toks(Vec<Tok>);

impl From<&WhereTokenizer> for Toks {
    fn from(b: &WhereTokenizer) -> Self {
        let mut t = Self::default();
        match b {
            WhereTokenizer::Binary(b) => b.as_param(&mut t),
            // we don't use Expr.as_param, to keep separated the behavior at first node.
            WhereTokenizer::Mono(m) => match m.as_ref() {
                // if only parenthesis, its first parsed as a group
                syn::Expr::Group(syn::ExprGroup { expr, .. }) => match **expr {
                    syn::Expr::Paren(ref p) => p.as_param(&mut t),
                    syn::Expr::Range(ref p) => p.as_param(&mut t),
                    _ => t.error(m, "Only Binary, Parenthesis and Not expression supported"),
                },
                syn::Expr::Range(ref p) => p.as_param(&mut t),
                syn::Expr::Paren(ref p) => p.as_param(&mut t),
                syn::Expr::Unary(ref p) => p.as_param(&mut t),
                _ => t.error(m, "Only Binary, Parenthesis and  Not expression supported"),
            },
        };
        t
    }
}

impl Toks {
    pub fn field(&mut self, ident: &syn::Ident) {
        self.0.push(Tok::Field(ident.clone()))
    }

    pub fn foreign_key(&mut self, field: &syn::ExprField) {
        self.0.push(Tok::ForeignKey(field.clone()))
    }
    pub fn iin(&mut self, toks: &Toks) {
        self.0.push(Tok::In(toks.clone()))
    }

    // pub fn call(&mut self, expr: &syn::Expr) {
    //     self.0.push(Tok::Call(expr.clone()))
    // }
    pub fn not(&mut self, toks: &Toks) {
        self.0.push(Tok::Not(toks.clone()))
    }

    pub fn null(&mut self, expr: &syn::Expr, op: &syn::BinOp) {
        let mut toks = Toks::default();
        expr.as_param(&mut toks);
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

impl Display for Toks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(|t| t.to_string()).collect::<String>()
        )
    }
}

#[derive(Debug, Clone)]
pub enum Tok {
    // Call(syn::Expr),
    Field(syn::Ident),
    ForeignKey(syn::ExprField),
    In(Toks),
    Null(Toks),
    Not(Toks),
    Paren(Toks),
    Sign(String),
    Value(syn::Expr),
    Error(syn::Error),
}

impl Display for Tok {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(i) => write!(f, "{}", i),
            Self::ForeignKey(p) => match *p.base {
                syn::Expr::Path(ref path) => {
                    let base = path.path.get_ident().unwrap();
                    let attre = match p.member {
                        syn::Member::Named(ref x) => x.to_string(),
                        syn::Member::Unnamed(ref x) => x.index.to_string(),
                    };
                    write!(f, "{}.{}", base, attre)
                }
                _ => unimplemented!(),
            },
            Self::Sign(s) => write!(f, "{}", s),
            Self::Value(v) => write!(f, "{}", display_expr(v)),
            Self::Error(e) => write!(f, "{}", e),
            Self::In(i) => {
                let mut iter = i.clone().into_iter();
                write!(f, "{} in ({})", &iter.next().unwrap(), &iter.join(","))
            }
            Self::Null(t) => write!(f, "{}None", t),
            Self::Not(n) => write!(f, "!{}", &n.to_string()),
            // Self::Call(n) => write!(f, "{}", display_expr(n)),
            Self::Paren(p) => write!(f, "({})", &p.to_string()),
        }
    }
}
