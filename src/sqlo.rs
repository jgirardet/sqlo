use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use crate::{field::Field, parse::SqloParse, serdable::IdentSer, types::is_type_option};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

const DATABASE_TYPE: DatabaseType = if cfg!(feature = "sqlite") {
    DatabaseType::Sqlite
} else {
    panic!(
        "You need to specify db backend as feature to use Sqlo. Right now only `sqlite` is
    suppported, PR Welcomed :-)"
    )
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Sqlo {
    #[serde(with = "IdentSer")]
    pub ident: syn::Ident,
    pub fields: Vec<Field>,
    pub tablename: String,
    pub database_type: DatabaseType,
    pub pk_field: Field,
}

impl TryFrom<SqloParse> for Sqlo {
    type Error = syn::Error;
    fn try_from(sp: SqloParse) -> Result<Sqlo, syn::Error> {
        Ok(Self {
            ident: sp.ident.clone(),
            fields: sp.fields()?,
            tablename: sp.tablename(),
            database_type: DATABASE_TYPE,
            pk_field: sp.has_pk_field()?,
        })
    }
}

impl Sqlo {
    pub fn fields_name_and_type_as_option(&self) -> TokenStream {
        self.fields
            .iter()
            .map(|Field { ident, ty, .. }| {
                if is_type_option(&ty) {
                    quote! {#ident: #ty,}
                } else {
                    quote! { #ident: Option<#ty>, }
                }
            })
            .collect()
    }
    pub fn all_columns_as_query(&self) -> String {
        self.fields.iter().map(|x| x.as_query.as_str()).join(",")
    }

    pub fn as_option_struct(&self) -> (syn::Ident, TokenStream) {
        let option_class = format_ident!("Option{}", &self.ident);
        let option_struct_name = option_class.clone();
        let class_args = self.fields_name_and_type_as_option();
        (
            option_struct_name,
            quote! {
                struct #option_class {
                    #class_args
                }
            },
        )
    }

    // Check for null values on Option_struct when using  `RETURNING`
    // and return the corresponding strut
    // called as tuple to not forget sqlx_null_checks.
    pub fn convert_struct_option_to_struct(&self) -> (TokenStream, TokenStream) {
        let sqlx_null_checks = self
            .fields
            .iter()
            .map(|x| {
                let ident = x.ident.clone();
                if !is_type_option(&x.ty) {
                    return quote! {
                    if res.#ident.is_none() {return Err(sqlx::Error::RowNotFound)}};
                }
                return quote! {};
            })
            .collect::<TokenStream>();

        let key_values = self
            .fields
            .iter()
            .map(|crate::field::Field { ident, ty, .. }| {
                if is_type_option(ty) {
                    return quote! {#ident:res.#ident,};
                }
                return quote! {#ident:res.#ident.unwrap(),}; //unwrap ok because check in sqlx_null_check
            })
            .collect::<TokenStream>();
        let struct_ident = &self.ident;
        (sqlx_null_checks, quote! [ #struct_ident{#key_values}])
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DatabaseType {
    Sqlite,
}

impl Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Sqlite => write!(f, "Sqlite"),
        }
    }
}

impl ToTokens for DatabaseType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;
        let name = format_ident!("{self}");
        tokens.append(name);
    }
}

impl DatabaseType {
    pub fn get_qmark(&self) -> &str {
        match self {
            DatabaseType::Sqlite => "?",
        }
    }
}

impl FromStr for DatabaseType {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sqlite" => Ok(DatabaseType::Sqlite),
            _ => Ok(DatabaseType::Sqlite),
        }
    }
}
