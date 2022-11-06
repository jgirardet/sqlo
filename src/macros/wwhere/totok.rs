use darling::ToTokens;
use syn::{BinOp, Expr};

use super::{
    tok::Toks,
    tokenizer::{parse_binary_bool, parse_binary_comp, parse_binary_eq},
};

pub(crate) trait ToTok
where
    Self: ToTokens,
{
    fn as_param(&self, acc: &mut Toks) {
        acc.error(&self, "Not supported as parameter")
    }
    fn as_value(&self, acc: &mut Toks) {
        acc.error(&self, "Not supported as value")
    }
}

impl ToTok for syn::Expr {
    fn as_param(&self, acc: &mut Toks) {
        match self {
            Expr::Path(p) => p.as_param(acc),
            Expr::Field(f) => f.as_param(acc),
            // Expr::Call(c) => c.as_param(acc),
            // Expr::Lit(l) => l.to_tok_left(acc),
            _ => {
                acc.error(self, "Not supported as parameter");
                return;
            }
        }
    }

    fn as_value(&self, acc: &mut Toks) {
        match self {
            Expr::Array(_)
            | Expr::Field(_)
            | Expr::Index(_)
            | Expr::Lit(_)
            // | Expr::Reference(_) //doesn't work either with sqlx
            | Expr::Tuple(_) => acc.value(self),
            // | Expr::MethodCall(_)
            // | Expr::Call(_)
            Expr::Path(p) => p.as_value(acc),
            // Expr::Unary(u) => u.to_tok_right(acc),
            _ => acc.error(self, "Not supported as rhs of comparison expression"),
        }
    }
}

/// This is a special case, which acts more to dispatch
impl ToTok for syn::ExprBinary {
    fn as_param(&self, acc: &mut Toks) {
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

    fn as_value(&self, acc: &mut Toks) {
        self.as_param(acc)
    }
}
// impl ToTok for syn::ExprCall {
//     fn as_param(&self, acc: &mut Toks) {
//         acc.call(&Expr::Call(self.clone()))
//     }

//     fn as_value(&self, acc: &mut Toks) {
//         acc.error(&self, "Not supported as value")
//     }
// }

// impl ToTok for syn::ExprMethodCall {
//     fn as_param(&self, acc: &mut Toks) {
//         acc.call(&Expr::MethodCall(self.clone()))
//     }

//     fn as_value(&self, acc: &mut Toks) {
//         acc.error(&self, "Not supported as value")
//     }
// }

impl ToTok for syn::ExprField {
    fn as_param(&self, acc: &mut Toks) {
        acc.foreign_key(self)
    }

    fn as_value(&self, acc: &mut Toks) {
        acc.error(self, "Can't be used on right side of binary expression")
    }
}

impl ToTok for syn::ExprLit {
    fn as_value(&self, acc: &mut Toks) {
        acc.value(&syn::Expr::Lit(self.clone()))
    }
}

impl ToTok for syn::ExprParen {
    fn as_param(&self, acc: &mut Toks) {
        let mut paren = Toks::default();
        match *self.expr {
            Expr::Binary(ref b) => {
                b.as_param(&mut paren);
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

    fn as_value(&self, acc: &mut Toks) {
        self.as_param(acc)
    }
}

impl ToTok for syn::ExprPath {
    fn as_param(&self, acc: &mut Toks) {
        if let Some(ident) = self.path.get_ident() {
            acc.field(ident);
        }
    }

    fn as_value(&self, acc: &mut Toks) {
        let e: Expr = self.clone().into();
        acc.value(&e);
    }
}

// Comment utiliser le Unaray<!> ?
// - cas du None:
// - Reste:
//  - doit englober un binary entièrer pour la nier
//  - donc forcément entre un and/or donc pas ==/!=
//  - donc c comme un  Mono
//  - donc doit accepter un Toks
impl ToTok for syn::ExprUnary {
    fn as_param(&self, acc: &mut Toks) {
        let mut toks = Toks::default();
        match *self.expr {
            // Expr::Call(ref c) => c.as_param(&mut toks),
            // Expr::MethodCall(ref m) => m.as_param(&mut toks),
            Expr::Paren(ref p) => p.as_param(&mut toks),
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

    fn as_value(&self, acc: &mut Toks) {
        self.as_param(acc)
    }
}
