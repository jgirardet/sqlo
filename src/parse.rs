use crate::field::{Field, FieldParser};
use darling::FromDeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sqlo), supports(struct_named))]
pub struct SqloParse {
    pub ident: syn::Ident,
    data: darling::ast::Data<(), FieldParser>,
    tablename: Option<String>,
    #[darling(default)]
    pub parse_only: bool,
}

// parser methods
impl SqloParse {
    pub fn has_pk_field(&self) -> syn::Result<Field> {
        let fields = self.fields()?;
        for f in &fields {
            if f.primary_key {
                return Ok(f.clone());
            }
        }
        for f in &fields {
            if f.ident == "id" {
                return Ok(f.clone());
            }
        }
        Err(syn::Error::new(
            self.ident.span(),
            "Sqlo should have one field with attirbute `primary_key` or an `id` field!",
        ))
    }

    pub fn tablename(&self) -> String {
        if let Some(ref tablename) = self.tablename {
            tablename.to_string()
        } else {
            heck::AsSnakeCase(self.ident.to_string()).to_string()
        }
    }

    pub fn fields(&self) -> syn::Result<Vec<Field>> {
        self.data
            .clone()
            .take_struct()
            .expect("should never fail")
            .fields
            .into_iter()
            .map(|f| f.try_into())
            .collect()
    }

    pub fn all_columns_as_query(fields: &[Field], tablename: &str) -> String {
        let mut res = vec![];
        for f in fields.iter() {
            // we write full query if name or type isn't the same between rust struct and database
            if f.type_override || f.ident != f.column || f.ident == "id" {
                let a =
                    format!(r#"{}.{} as "{}:_""#, tablename, &f.column, &f.ident).replace('\\', "");
                res.push(a);
            } else {
                let a = format!("{}.{}", tablename, f.column);
                res.push(a)
            }
        }
        res.join(", ")
    }
}
