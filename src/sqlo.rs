use std::{fmt::Display, str::FromStr};

use crate::{field::Field, parse::SqloParse, serdable::IdentStringSer, types::is_type_option};
use darling::util::IdentString;
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};

const DATABASE_TYPE: DatabaseType = if cfg!(feature = "sqlite") {
    DatabaseType::Sqlite
} else {
    panic!(
        "You need to specify db backend as feature to use Sqlo. Right now only `sqlite` is
    suppported, PR Welcomed :-)"
    )
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sqlo {
    #[serde(with = "IdentStringSer")]
    pub ident: IdentString,
    pub fields: Vec<Field>,
    pub tablename: String,
    pub database_type: DatabaseType,
    pub pk_field: Field,
    pub parse_only: bool,
    pub all_columns_as_query: String,
}

impl TryFrom<SqloParse> for Sqlo {
    type Error = syn::Error;
    fn try_from(sp: SqloParse) -> Result<Sqlo, syn::Error> {
        let tablename = sp.tablename();
        let fields = sp.fields()?;
        let all_columns_as_query = SqloParse::all_columns_as_query(fields.as_slice(), &tablename);
        Ok(Self {
            fields,
            tablename,
            pk_field: sp.has_pk_field()?,
            ident: sp.ident.into(),
            database_type: DATABASE_TYPE,
            parse_only: sp.parse_only,
            all_columns_as_query,
        })
    }
}

impl Sqlo {
    pub fn to_non_null_columns(&self) -> String {
        let mut res = vec![];
        for field in &self.fields {
            if !is_type_option(&field.ty) {
                res.push(format!("{} as \"{}!:_\"", &field.column, &field.ident))
            } else {
                res.push(field.column.clone())
            }
        }
        res.join(",")
    }
}

// utils
impl Sqlo {
    /// Get a field if exists.
    pub fn field(&self, name: &syn::Ident) -> Option<&Field> {
        self.fields.iter().find(|f| f.ident.as_ident() == name)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
