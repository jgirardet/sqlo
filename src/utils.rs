use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::BinOp;

pub fn compile_error<T: ToTokens, U: Display>(tokens: T, message: U) -> TokenStream {
    syn::Error::new_spanned(tokens, message).to_compile_error()
}

pub fn op_to_str(op: &BinOp) -> &str {
    match op {
        BinOp::Eq(_) => "==",
        BinOp::Ne(_) => "!=",
        BinOp::Le(_) => "<=",
        BinOp::Lt(_) => "<",
        BinOp::Ge(_) => ">=",
        BinOp::Gt(_) => ">",
        BinOp::And(_) => "&&",
        BinOp::Or(_) => "||",
        BinOp::Add(_) => "+",
        BinOp::Sub(_) => "-",
        BinOp::Mul(_) => "*",
        BinOp::Div(_) => "/",
        _ => unimplemented!("Sign to str not supported"),
    }
}
