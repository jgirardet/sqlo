use crate::serdable::{IdentSer, OptionExprPathSer, TypeSer};
use darling::FromField;
use syn::spanned::Spanned;

#[derive(Debug, FromField, Clone)]
#[darling(attributes(sqlo))]
pub struct FieldParser {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    #[darling(default)]
    pub type_override: bool,
    #[darling(default)]
    pub primary_key: bool,
    #[darling(default)]
    column: Option<String>,
    create_fn: Option<syn::ExprPath>,
    #[darling(default)]
    pub create_arg: bool,
}

impl FieldParser {
    pub fn ident(&self) -> syn::Result<&syn::Ident> {
        if let Some(ref ident) = self.ident {
            return Ok(ident);
        }
        Err(syn::Error::new(
            self.ty.span(),
            "Use Sqlo anly with struct name",
        ))
    }

    pub fn column_name<'a>(&'a self) -> syn::Result<String> {
        if let Some(ref nom) = self.column {
            return Ok(nom.to_string());
        }
        Ok(self.ident().unwrap().to_string())
    }

    pub fn as_query(&self) -> syn::Result<String> {
        let name = self.column_name()?;
        let struct_name = self.ident()?;
        // we write full query if name or type isn't the same between rust struct and database
        if self.type_override || name != struct_name.to_string() || struct_name == "id" {
            Ok(format!(r#"{} as "{}:_""#, &name, &struct_name).replace("\\", ""))
        } else {
            Ok(name.to_string())
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    #[serde(with = "IdentSer")]
    pub ident: syn::Ident,
    #[serde(with = "TypeSer")]
    pub ty: syn::Type,
    pub column: String,
    pub as_query: String,
    pub primary_key: bool,
    #[serde(with = "OptionExprPathSer")]
    pub create_fn: Option<syn::ExprPath>,
    pub create_arg: bool,
}

impl<'a> TryFrom<FieldParser> for Field {
    type Error = syn::Error;

    fn try_from(fp: FieldParser) -> Result<Self, Self::Error> {
        Ok(Field {
            ident: fp.ident()?.to_owned(),
            ty: fp.ty.clone(),
            column: fp.column_name()?.to_string(),
            as_query: fp.as_query()?,
            primary_key: fp.primary_key,
            create_fn: fp.create_fn,
            create_arg: fp.create_arg,
        })
    }
}
