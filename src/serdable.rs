use darling::util::IdentString;
use quote::ToTokens;

// Serde Adaptor for various type

// syn::Ident

// syn::Ident

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "darling::util::IdentString")]
pub(crate) struct IdentStringSer {
    #[serde(getter = "darling::util::IdentString::to_string")]
    name: String,
}

impl From<IdentStringSer> for darling::util::IdentString {
    fn from(i: IdentStringSer) -> Self {
        darling::util::IdentString::new(syn::Ident::new(&i.name, proc_macro2::Span::call_site()))
    }
}

// darlug::util::IdentString

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "syn::Ident")]
pub(crate) struct IdentSer {
    #[serde(getter = "syn::Ident::to_string")]
    name: String,
}

impl From<IdentSer> for syn::Ident {
    fn from(i: IdentSer) -> Self {
        syn::Ident::new(&i.name, proc_macro2::Span::call_site())
    }
}

// Option<syn::Ident>

// #[derive(serde::Serialize, serde::Deserialize)]
// #[serde(remote = "Option<syn::Ident>")]
// pub(crate) struct OptionIdentSer {
//     #[serde(getter = "option_ident_to_string")]
//     name: Option<String>,
// }

// impl From<OptionIdentSer> for Option<syn::Ident> {
//     fn from(i: OptionIdentSer) -> Self {
//         i.name
//             .map(|x| syn::Ident::new(&x, proc_macro2::Span::call_site()))
//     }
// }
// fn option_ident_to_string(exp: &Option<syn::Ident>) -> Option<String> {
//     exp.as_ref().map(|p| p.to_string())
// }

// Option<IdentString>

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Option<darling::util::IdentString>")]
pub(crate) struct OptionIdentStringSer {
    #[serde(getter = "option_ident_string_to_string")]
    name: Option<String>,
}

impl From<OptionIdentStringSer> for Option<IdentString> {
    fn from(i: OptionIdentStringSer) -> Self {
        i.name
            .map(|x| syn::Ident::new(&x, proc_macro2::Span::call_site()).into())
    }
}
fn option_ident_string_to_string(exp: &Option<IdentString>) -> Option<String> {
    exp.as_ref().map(|p| p.to_string())
}

// // syn::Type

// #[derive(serde::Serialize, serde::Deserialize)]
// #[serde(remote = "syn::Type")]
// pub(crate) struct TypeSer {
//     #[serde(getter = "type_to_string")]
//     ty: String,
// }

// fn type_to_string(typ: &syn::Type) -> String {
//     typ.to_token_stream().to_string()
// }

// impl From<TypeSer> for syn::Type {
//     fn from(i: TypeSer) -> Self {
//         syn::parse_str::<syn::Type>(&i.ty).expect("Error deserializing syn::Type")
//     }
// }

// syn::TypePath

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "syn::TypePath")]
pub(crate) struct TypePathSer {
    #[serde(getter = "type_to_string")]
    ty: String,
}

fn type_to_string(typ: &syn::TypePath) -> String {
    typ.to_token_stream().to_string()
}

impl From<TypePathSer> for syn::TypePath {
    fn from(i: TypePathSer) -> Self {
        syn::parse_str::<syn::TypePath>(&i.ty).expect("Error deserializing syn::TypePath")
    }
}

// // syn::ExprPath

// #[derive(serde::Serialize, serde::Deserialize)]
// #[serde(remote = "syn::ExprPath")]
// pub(crate) struct ExprPathSer {
//     #[serde(getter = "expr_path_to_string")]
//     path: String,
// }

// impl From<ExprPathSer> for syn::ExprPath {
//     fn from(i: ExprPathSer) -> Self {
//         syn::parse_str::<syn::ExprPath>(&i.path).expect("Error deserializing syn::ExprPath")
//     }
// }

// fn expr_path_to_string(path: &syn::ExprPath) -> String {
//     darling::util::path_to_string(&path.path)
// }

// Option<syn::ExprPath>

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Option<syn::ExprPath>")]
pub(crate) struct OptionExprPathSer {
    #[serde(getter = "option_expr_path_to_string")]
    path: String,
}

fn option_expr_path_to_string(exp: &Option<syn::ExprPath>) -> String {
    if let Some(p) = exp {
        darling::util::path_to_string(&p.path)
    } else {
        "".to_string()
    }
}

impl From<OptionExprPathSer> for Option<syn::ExprPath> {
    fn from(i: OptionExprPathSer) -> Self {
        match i.path.as_str() {
            "" => None,
            x => {
                Some(syn::parse_str::<syn::ExprPath>(x).expect("Error deserializing syn::ExprPath"))
            }
        }
    }
}

// Option<syn::Path>

// #[derive(serde::Serialize, serde::Deserialize)]
// #[serde(remote = "Option<syn::Path>")]
// pub(crate) struct OptionPathSer {
//     #[serde(getter = "option_path_to_string")]
//     path: String,
// }

// fn option_path_to_string(exp: &Option<syn::Path>) -> String {
//     if let Some(p) = exp {
//         darling::util::path_to_string(&p)
//     } else {
//         "".to_string()
//     }
// }

// impl From<OptionPathSer> for Option<syn::Path> {
//     fn from(i: OptionPathSer) -> Self {
//         match i.path.as_str() {
//             "" => None,
//             x => Some(syn::parse_str::<syn::Path>(&x).expect("Error deserializing syn::Path")),
//         }
//     }
// }
