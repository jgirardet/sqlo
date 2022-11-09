use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, Lit, Member};

use darling::util::path_to_string;

pub fn compile_error<T: ToTokens, U: Display>(tokens: T, message: U) -> TokenStream {
    syn::Error::new_spanned(tokens, message).to_compile_error()
}
/// Turn syn::Expr to a Humane readable format.
pub fn display_expr(expr: &Expr) -> String {
    let mut acc = String::new();
    print_expr(expr, &mut acc);
    acc
}

fn print_expr(expr: &Expr, acc: &mut String) {
    match expr {
        Expr::Path(path) => acc.push_str(&path_to_string(&path.path)),
        Expr::Field(field) => {
            print_expr(&field.base, acc);
            acc.push('.');
            match field.member {
                Member::Named(ref n) => acc.push_str(&n.to_string()),
                Member::Unnamed(ref i) => acc.push_str(&i.index.to_string()),
            }
        }
        Expr::Array(a) => {
            acc.push('[');
            for it in a.elems.iter() {
                print_expr(it, acc);
                acc.push(',');
            }
            if acc.ends_with(',') {
                acc.pop();
            } //remove laste comma
            acc.push(']');
        }
        Expr::Index(i) => {
            print_expr(&i.expr, acc);
            acc.push('[');
            print_expr(&i.index, acc);
            acc.push(']');
        }
        Expr::Lit(l) => match l.lit {
            Lit::Bool(ref b) => acc.push_str(&b.value().to_string()),
            Lit::Str(ref s) => acc.push_str(&s.value()),
            Lit::Int(ref i) => acc.push_str(i.base10_digits()),
            Lit::Float(ref f) => acc.push_str(f.base10_digits()),
            _ => unimplemented!("Print of some Expr literal"),
        },
        Expr::MethodCall(m) => {
            print_expr(&m.receiver, acc);
            acc.push('.');
            acc.push_str(&m.method.to_string());
            acc.push('(');
            for it in m.args.iter() {
                print_expr(it, acc);
                acc.push(',')
            }
            if acc.ends_with(',') {
                acc.pop();
            } //remove laste comma
            acc.push(')');
        }
        Expr::Call(m) => {
            print_expr(&m.func, acc);
            acc.push('(');
            for it in m.args.iter() {
                print_expr(it, acc);
                acc.push(',')
            }
            if acc.ends_with(',') {
                acc.pop();
            } //remove laste comma
            acc.push(')');
        }
        Expr::Reference(r) => {
            acc.push('&');
            print_expr(&r.expr, acc);
        }
        Expr::Tuple(t) => {
            acc.push('(');
            for it in t.elems.iter() {
                print_expr(it, acc);
                acc.push(',');
            }
            if acc.ends_with(',') {
                acc.pop();
            } //remove laste comma
            acc.push(')');
        }
        Expr::Unary(u) => {
            acc.push('!');
            print_expr(&u.expr, acc)
        }
        _ => unimplemented!("Print of some Expr"),
    }
}
