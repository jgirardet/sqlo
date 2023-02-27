use darling::ToTokens;
use syn::{parse2, BinOp, Expr, ExprRange};

use super::{
    tok::Toks,
    tokenizer::{parse_binary_bool, parse_binary_comp, parse_binary_eq},
    Like,
};

pub(crate) trait ToTok
where
    Self: ToTokens,
{
    fn to_tok(&self, acc: &mut Toks);
}

impl ToTok for syn::Expr {
    fn to_tok(&self, acc: &mut Toks) {
        match self {
            Expr::Array(_) | Expr::Lit(_) | Expr::Tuple(_) => acc.value(self),
            Expr::Index(i) => i.to_tok(acc),
            Expr::Field(f) => f.to_tok(acc),
            Expr::Path(p) => p.to_tok(acc),
            _ => acc.error(self, "Not supported as rhs of comparison expression"),
        }
    }
}

/// This is a special case, which acts more to dispatch
impl ToTok for syn::ExprBinary {
    fn to_tok(&self, acc: &mut Toks) {
        let syn::ExprBinary {
            left, op, right, ..
        } = self;
        match op {
            BinOp::Eq(_) | BinOp::Ne(_) => parse_binary_eq(left.as_ref(), op, right.as_ref(), acc),
            BinOp::Lt(_) | BinOp::Le(_) | BinOp::Ge(_) | BinOp::Gt(_) => {
                parse_binary_comp(left.as_ref(), op, right.as_ref(), acc)
            }

            BinOp::And(_) | BinOp::Or(_) => {
                parse_binary_bool(left.as_ref(), op, right.as_ref(), acc)
            }
            _ => acc.error(op, "Operator not permitted"),
        }
    }
}

impl ToTok for syn::ExprIndex {
    fn to_tok(&self, acc: &mut Toks) {
        if let Expr::Path(p) = self.expr.as_ref() {
            if p.path.leading_colon.is_some() {
                let mut index2 = self.clone();
                if let Expr::Path(ref mut p) = index2.expr.as_mut() {
                    p.path.leading_colon = None
                }
                acc.value(&index2.into());
                return;
            }
        }
        acc.error(self, "Rust values must be prefixed by `::`")
    }
}

impl ToTok for syn::ExprField {
    fn to_tok(&self, acc: &mut Toks) {
        if let Expr::Path(p) = self.base.as_ref() {
            if p.path.leading_colon.is_some() {
                let mut p2 = p.clone();
                p2.path.leading_colon = None;
                let mut field2 = self.clone();
                field2.base = Box::new(p2.into());
                acc.value(&field2.into());
                return;
            } else if p.path.get_ident().is_some() {
                acc.foreign_key(self);
                return;
            }
        }

        acc.error(self, "Rust values must be prefixed by `::`")
    }
}

impl ToTok for syn::ExprMacro {
    fn to_tok(&self, acc: &mut Toks) {
        let mac = &self.mac;
        if let Some(p) = mac.path.get_ident() {
            if p == "like" {
                match parse2::<Like>(mac.tokens.clone()) {
                    Ok(like) => acc.like(like),
                    Err(_) => acc.error(&mac.tokens, "Invalid content"),
                }
            } else {
                acc.error(&mac.path, "only suppported macros are: like")
            }
        } else {
            acc.error(&mac.path, "macros call is single word")
        }
    }
}

impl ToTok for syn::ExprLit {
    fn to_tok(&self, acc: &mut Toks) {
        acc.value(&syn::Expr::Lit(self.clone()))
    }
}

impl ToTok for syn::ExprParen {
    fn to_tok(&self, acc: &mut Toks) {
        let mut paren = Toks::default();
        match *self.expr {
            Expr::Binary(ref b) => {
                b.to_tok(&mut paren);
            }
            _ => {
                acc.error(
                    self,
                    "Parenthesized expression only supports binary,call or method expression",
                );
                return;
            }
        }
        acc.paren(&paren)
    }
}

impl ToTok for syn::ExprPath {
    fn to_tok(&self, acc: &mut Toks) {
        if self.path.leading_colon.is_some() {
            if self.path.segments.len() == 1 {
                if let Some(_) = self.path.segments.first() {
                    let mut path2 = self.clone();
                    path2.path.leading_colon = None;
                    acc.value(&path2.clone().into());
                    return;
                }
            } else {
                acc.error(self, "use only the field name")
            }
            return;
        } else if let Some(ident) = self.path.get_ident() {
            acc.field(ident);
            return;
        }
        acc.error(self, "rust var must be prefixed by `::`")
    }
}

impl ToTok for syn::ExprRange {
    fn to_tok(&self, acc: &mut Toks) {
        let mut toks = Toks::default();
        if let Some(ref from) = self.from {
            // get the column
            from.to_tok(&mut toks);
            if let Some(ref to) = self.to {
                match to.as_ref() {
                    // a..[1,2,3]
                    syn::Expr::Array(a) => {
                        for v in &a.elems {
                            v.to_tok(&mut toks);
                        }
                        acc.iin(&toks);
                        return;
                    }
                    // a..(1,2,3)
                    syn::Expr::Tuple(a) => {
                        for v in &a.elems {
                            v.to_tok(&mut toks);
                        }
                        acc.iin(&toks);
                        return;
                    }
                    // a..(1..2)
                    syn::Expr::Paren(ref p) => {
                        if let syn::Expr::Range(r) = p.expr.as_ref() {
                            // r.to_tok(&mut toks);
                            range_as_value(r, &mut toks);
                            acc.iin(&toks);
                            return;
                        }
                    }
                    _ => acc.error(&self, "Expression not supported"),
                }
            }
        }

        acc.error(self, "In Sql use range with column as start and exp as end")
    }
}

fn range_as_value(expr: &ExprRange, acc: &mut Toks) {
    if let Some(b) = expr.from.as_ref() {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(l),
            ..
        }) = b.as_ref()
        {
            let from = l.base10_parse::<usize>().unwrap();

            if let Some(b) = expr.to.as_ref() {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(l),
                    ..
                }) = b.as_ref()
                {
                    let mut to = l.base10_parse::<usize>().unwrap();
                    if let syn::RangeLimits::Closed(_) = expr.limits {
                        to += 1;
                    }
                    for v in from..to {
                        let exp: syn::Expr = syn::ExprLit {
                            lit: syn::Lit::new(proc_macro2::Literal::usize_unsuffixed(v)),
                            attrs: vec![],
                        }
                        .into();
                        exp.to_tok(acc)
                    }
                    return;
                }
            }
        }
    }
    acc.error(expr, "this range is not a valid input")
}

// Comment utiliser le Unaray<!> ?
// - cas du None:
// - Reste:
//  - doit englober un binary entièrer pour la nier
//  - donc forcément entre un and/or donc pas ==/!=
//  - donc c comme un  Mono
//  - donc doit accepter un Toks
impl ToTok for syn::ExprUnary {
    fn to_tok(&self, acc: &mut Toks) {
        let mut toks = Toks::default();
        match *self.expr {
            Expr::Paren(ref p) => p.to_tok(&mut toks),
            _ => {
                acc.error(
                    &self.expr,
                    "Not supported with `!` operator. Did you forget parenthesis ?",
                );
                return;
            }
        }

        acc.not(&toks)
    }
}
