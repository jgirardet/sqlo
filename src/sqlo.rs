use crate::{field::Field, parse::SqloParse, serdable::IdentStringSer, types::is_type_option};
use darling::util::IdentString;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sqlo {
    #[serde(with = "IdentStringSer")]
    pub ident: IdentString,
    pub fields: Vec<Field>,
    pub tablename: String,
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
