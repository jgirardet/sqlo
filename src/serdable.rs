use darling::util::IdentString;
use quote::ToTokens;

// Serde Adaptor for various type

// syn::Ident
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

// darling::util::IdentString
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

// Option<syn::ExprPath>
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Option<syn::ExprPath>")]
pub(crate) struct OptionExprPathSer {
    #[serde(getter = "option_expr_path_to_string")]
    path: Option<String>,
}

fn option_expr_path_to_string(exp: &Option<syn::ExprPath>) -> Option<String> {
    exp.as_ref().map(|p| darling::util::path_to_string(&p.path))
}

impl From<OptionExprPathSer> for Option<syn::ExprPath> {
    fn from(i: OptionExprPathSer) -> Self {
        i.path.map(|x| {
            syn::parse_str::<syn::ExprPath>(&x).expect("Error deserializing syn::ExprPath")
        })
    }
}
