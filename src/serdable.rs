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

// syn::Type

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "syn::Type")]
pub(crate) struct TypeSer {
    #[serde(getter = "type_to_string")]
    ty: String,
}

fn type_to_string(typ: &syn::Type) -> String {
    typ.to_token_stream().to_string()
}

impl From<TypeSer> for syn::Type {
    fn from(i: TypeSer) -> Self {
        syn::parse_str::<syn::Type>(&i.ty).expect("Error deserializing syn::Type")
    }
}

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
            x => Some(
                syn::parse_str::<syn::ExprPath>(&x).expect("Error deserializing syn::ExprPath"),
            ),
        }
    }
}
