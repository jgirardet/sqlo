use itertools::Itertools;

pub fn parse_manifest() -> syn::Result<cargo_toml::Manifest> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "CARGO_MANIFEST_DIR env variable was not found... that's strange",
        )
    })?;
    let path: std::path::PathBuf = [manifest_dir, "Cargo.toml".to_string()]
        .into_iter()
        .collect();
    if !path.is_file() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "CARGO_MANIFEST_DIR env variable was not found... that's strange",
        ));
    }
    cargo_toml::Manifest::from_path(path).map_err(|_| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Fail reading Cargo.toml but the file exists.",
        )
    })
}

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

// trait sqlo_t
// add commas between sting element inside a seq
pub fn add_comma_between<'a, T>(seq: T) -> Vec<&'a str>
where
    T: IntoIterator<Item = &'a str>,
{
    Itertools::intersperse(seq.into_iter(), &",").collect::<Vec<&'a str>>()
}
