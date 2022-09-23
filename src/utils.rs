use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Path, Type, TypePath, TypeReference};

pub fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(p) = segments.first() {
            return p.ident == "Option";
        }
    }
    false
}

pub fn is_string_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(ident) = path.get_ident() {
                ident == "String"
            } else {
                false
            }
        }
        Type::Reference(TypeReference { elem, .. }) => {
            if let Type::Path(TypePath { ref path, .. }) = **elem {
                if let Some(ident) = path.get_ident() {
                    ident == "str"
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn compile_error<T: ToTokens, U: Display>(tokens: T, message: U) -> TokenStream {
    syn::Error::new_spanned(tokens, message).to_compile_error()
}
