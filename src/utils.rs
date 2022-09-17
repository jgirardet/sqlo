pub fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(p) = segments.first() {
            return p.ident == "Option";
        }
    }
    false
}
