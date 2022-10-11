use darling::util::path_to_string;
use proc_macro2::TokenStream;
use quote::quote;

pub fn get_function_arg_type(ty: &syn::TypePath) -> TokenStream {
    if let Some(ident) = ty.path.get_ident() {
        match ident.to_string().as_str() {
            "u8" => quote![u8],
            "u16" => quote![u16],
            "u32" => quote![u32],
            "u64" => quote![u64],
            "i8" => quote![i8],
            "i16" => quote![i16],
            "i32" => quote![i32],
            "i64" => quote![i64],
            "f32" => quote![f32],
            "f64" => quote![f64],
            "bool" => quote![bool],
            "String" => quote![&str],
            "BString" => quote![&BStr],
            "bstr::BString" => quote![&bstr::BStr],
            _ => quote!(&#ty),
        }
    } else {
        if path_is_vec_u8(&ty.path) {
            quote![&[u8]]
        } else if is_type_option(&ty) {
            quote![#ty]
        } else {
            match path_to_string(&ty.path).as_str() {
                "bstr::BString" => quote![&bstr::BStr],
                _ => {
                    let path = ty.path.clone();
                    quote![&#path]
                }
            }
        }
    }
}

fn path_is_vec_u8(path: &syn::Path) -> bool {
    if let Some(syn::PathSegment {
        ident, arguments, ..
    }) = path.segments.first()
    {
        if ident == "Vec" {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
            }) = arguments
            {
                if let Some(arg) = args.first() {
                    if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                        path, ..
                    })) = arg
                    {
                        if let Some(ident) = path.get_ident() {
                            if ident == "u8" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn is_type_option(ty: &syn::TypePath) -> bool {
    if let Some(p) = ty.path.segments.first() {
        p.ident == "Option"
    } else {
        false
    }
}

#[allow(unused)]
struct Bla;

#[allow(non_snake_case)]
#[cfg(test)]
mod test_types {
    use super::*;
    use syn::parse_quote;

    macro_rules! test_types {
        ($title:expr, $input:ty, $res:ty) => {
            paste::paste! {


                #[test]
                fn [<test_get_function_arg_type $title>]() {
                    let ty : syn::TypePath = parse_quote!($input);
                    assert_eq!(
                        get_function_arg_type(&ty).to_string(),
                        quote![$res].to_string()
                    )
                }
            }
        };
    }
    test_types![all, Bla, &Bla];
    test_types![u8, u8, u8];
    test_types![u16, u16, u16];
    test_types![u32, u32, u32];
    test_types![u64, u64, u64];
    test_types![i8, i8, i8];
    test_types![i16, i16, i16];
    test_types![i64, i64, i64];
    test_types![f32, f32, f32];
    test_types![f64, f64, f64];
    test_types![bool, bool, bool];
    test_types![String, String, &str];
    test_types![BString, BString, &BStr];
    test_types![crate_BString, bstr::BString, &bstr::BStr];
    test_types![Vecu8, Vec<u8>, &[u8]];
    test_types![Option_u8, Option<u8>, Option<u8>];
    test_types![Option_string, Option<String>, Option<String>];
    test_types![path_as_type, uuid::Uuid, &uuid::Uuid];
}
