use crate::serdable::{IdentStringSer, OptionExprPathSer, OptionIdentStringSer, TypePathSer};
use darling::{util::IdentString, FromField};
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
    pub fk: Option<IdentString>,
    pub related: Option<IdentString>,
}

impl FieldParser {
    pub fn ident(&self) -> syn::Result<IdentString> {
        if let Some(ref ident) = self.ident {
            return Ok(ident.clone().into());
        }
        Err(syn::Error::new(
            self.ty.span(),
            "Use Sqlo anly with struct name",
        ))
    }

    pub fn column_name(&self) -> syn::Result<String> {
        if let Some(ref nom) = self.column {
            return Ok(nom.to_string());
        }
        Ok(self.ident().unwrap().to_string())
    }

    pub fn as_query(&self) -> syn::Result<String> {
        let name = self.column_name()?;
        let struct_name = self.ident()?;
        // we write full query if name or type isn't the same between rust struct and database
        if self.type_override || struct_name != name || struct_name == "id" {
            Ok(format!(r#"{} as "{}:_""#, &name, &struct_name).replace('\\', ""))
        } else {
            Ok(name)
        }
    }

    pub fn fk(&self) -> syn::Result<Option<IdentString>> {
        if self.fk.is_none() {
            return Ok(self.fk.clone());
        }
        let msg =  "This type is not supported as Foreign Key with Sqlo. use `Ident` or `some::path::Ident` without Generic";
        if let syn::Type::Path(syn::TypePath { ref path, .. }) = self.ty {
            // validate single ident as type path
            if path.get_ident().is_some() {
                return Ok(self.fk.clone());
            }
            // validate mutli path as type_path without <>or()
            for seg in path.segments.iter() {
                match seg.arguments {
                    syn::PathArguments::None => continue,
                    _ => return Err(syn::Error::new_spanned(self.ty.clone(), msg)),
                };
            }
            return Ok(self.fk.clone());
        }
        Err(syn::Error::new_spanned(self.ty.clone(), msg))
    }

    pub fn ty(&self) -> syn::Result<syn::TypePath> {
        if let syn::Type::Path(ref typepath) = self.ty {
            Ok(typepath.clone())
        } else {
            Err(syn::Error::new_spanned(
                &self.ty,
                "Type not supported by sqlo",
            ))
        }
    }

    pub fn related(&self) -> syn::Result<Option<IdentString>> {
        if self.related.is_some() && self.fk.is_none() {
            Err(syn::Error::new_spanned(
                self.related.clone(),
                "`fk` has to be set",
            ))
        } else {
            Ok(self.related.clone())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    #[serde(with = "IdentStringSer")]
    pub ident: darling::util::IdentString,
    #[serde(with = "TypePathSer")]
    pub ty: syn::TypePath,
    pub column: String,
    pub as_query: String,
    pub primary_key: bool,
    #[serde(with = "OptionExprPathSer")]
    pub create_fn: Option<syn::ExprPath>,
    pub create_arg: bool,
    #[serde(with = "OptionIdentStringSer")]
    pub fk: Option<IdentString>,
    #[serde(with = "OptionIdentStringSer")]
    pub related: Option<IdentString>,
}

impl TryFrom<FieldParser> for Field {
    type Error = syn::Error;

    fn try_from(fp: FieldParser) -> Result<Self, Self::Error> {
        Ok(Field {
            ident: fp.ident()?.clone().into(),
            ty: fp.ty()?,
            column: fp.column_name()?,
            as_query: fp.as_query()?,
            primary_key: fp.primary_key,
            fk: fp.fk()?,
            related: fp.related()?,
            create_fn: fp.create_fn,
            create_arg: fp.create_arg,
        })
    }
}
