use crate::{
    field::{Field, FieldParser},
    sqlo::DatabaseType,
    utils::parse_manifest,
};
use darling::FromDeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sqlo), supports(struct_named))]
pub struct SqloParse {
    pub ident: syn::Ident,
    data: darling::ast::Data<(), FieldParser>,
    tablename: Option<String>,
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

    pub fn database_type(&self) -> syn::Result<DatabaseType> {
        let cargo_toml::Manifest { dependencies, .. } = parse_manifest()?;
        let sqlx_package = dependencies.get("sqlx");
        if let Some(cargo_toml::Dependency::Detailed(cargo_toml::DependencyDetail {
            features,
            ..
        })) = sqlx_package
        {
            if features.contains(&"sqlite".to_string()) {
                return Ok(DatabaseType::Sqlite);
            }
        }
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "sqlo is usable only with Sqlite backend. PR welcomed :-)",
        ))
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

        // if fields.len() == 1 {
        //     return Err(syn::Error::new(
        //         self.ident.span(),
        //         "Deriving Sqlo needs à least one field other than primary key field",
        //     ));
        // }
        // fields.into_iter().collect()
    }
}

// impl TryFrom<syn::DeriveInput> for Sqlo {
//     type Error = syn::Error;

//     fn try_from(di: syn::DeriveInput) -> Result<Self, Self::Error> {
//         if let syn::DeriveInput {
//             data:
//                 syn::Data::Struct(syn::DataStruct {
//                     fields, //: syn::Fields::Named(syn::FieldsNamed { named, .. }),
//                     ..
//                 }),
//             ident,
//             attrs,
//             ..
//         } = di
//         {
//             // let fields = vec![]; //Vec<Field> = named.iter().map(|f| Field::from(f)).collect();
//             let mut fieldsss = vec![];
//             for f in &fields {
//                 fieldsss.push(Field::from_field(f)?);
//             }
//             let attrs = SqloAttrs::from(attrs);
//             let tablename = Sqlo::set_table_name(&ident.to_string(), &attrs);

//             let sqlo = Sqlo {
//                 name: ident.to_string(),
//                 ident,
//                 tablename,
//                 fields: fieldsss,
//                 attrs,
//             };
//             sqlo.post_validation()?;
//             Ok(sqlo)
//         } else {
//             Err(syn::Error::new(
//                 proc_macro2::Span::call_site(),
//                 "Sqlo should be used only on Named Struct",
//             ))
//         }
//     }
// }
