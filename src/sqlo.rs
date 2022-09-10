use std::fmt::Display;

use crate::{field::Field, parse::SqloParse, utils::is_option};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

pub struct Sqlo {
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
            database_type: sp.database_type()?,
            pk_field: sp.has_pk_field()?,
        })
    }
}

impl Sqlo {
    pub fn fields_name_and_type_as_option(&self) -> TokenStream {
        self.fields
            .iter()
            .map(|Field { ident, ty, .. }| {
                if is_option(&ty) {
                    quote! {#ident: #ty,}
                } else {
                    quote! { #ident: Option<#ty>, }
                }
            })
            .collect()
    }
    pub fn all_columns_as_query(&self) -> String {
        self.fields.iter().map(|x| x.as_query.as_str()).join("'")
    }
}

#[derive(Debug)]
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
            Sqlite => "?",
        }
    }
}
