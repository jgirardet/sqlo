use darling::util::path_to_string;
use proc_macro2::Span;
use regex::Regex;

use std::{
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use crate::{
    error::{SqloError, ToSqloError},
    field::Field,
    sqlo::Sqlo,
};

// not const so we can use it in macro
macro_rules! fk_pattern {
    () => {
        "fk-{}--{}---{}----{}"
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Relations {
    relations: Vec<Relation>,
}

impl Relations {
    /// Build new Relations from a Sqlo instance.
    /// Span is preserved for field
    pub fn from_sqlo(sqlo: &Sqlo) -> Relations {
        Relations {
            // base: sqlo.ident.to_string(),
            relations: sqlo
                .fields
                .iter()
                .map(|f| make_field_relations(f, sqlo))
                .flatten()
                .collect(),
        }
    }

    /// get all relations present in a dir
    /// No span is preserved
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let mut relations = vec![];

        // foreignkeys
        let pat = path.join(format!("fk-*--*--*"));
        let fk_relations: Vec<PathBuf> = glob::glob(pat.to_str().unwrap())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.msg))?
            .filter_map(Result::ok)
            .collect();
        relations.extend_from_slice(&fk_relations);

        Ok(Self {
            relations: relations
                .into_iter()
                .filter_map(|f| Relation::try_from(f).ok())
                .collect(),
        })
    }

    /// Turn relation to a Vec of Relation using thier PathBuf representation.
    pub fn to_files(&self) -> Vec<PathBuf> {
        self.relations
            .iter()
            .map(|r| PathBuf::from(r.to_string()))
            .collect()
    }

    #[allow(irrefutable_let_patterns)]
    pub fn filter_entity(&self, mode: &str, ident: &str) -> Relations {
        Relations {
            relations: self
                .relations
                .clone()
                .into_iter()
                .filter(|e| {
                    if let Relation::ForeignKey(f) = e {
                        if mode == "from" {
                            f.from == ident
                        } else if mode == "to" {
                            f.to == ident
                        } else if mode == "both" {
                            f.from == ident || f.to == ident
                        } else {
                            unreachable!("Only `to` and `from` are allowed")
                        }
                    } else {
                        false
                    }
                })
                .collect(),
        }
    }

    /// Returns PathPub of other not contained in self.
    pub fn difference_of_other(&self, other: &[PathBuf]) -> Vec<PathBuf> {
        let list = self.to_files();
        other
            .into_iter()
            .filter(|x| !list.contains(x))
            .cloned()
            .collect()
    }

    pub fn validate(&self, sqlo: &Sqlo, path: &Path) -> Result<(), SqloError> {
        let files = std::fs::read_dir(path).sqlo_err(sqlo.ident.span())?;
        let mut sqlos: Vec<Sqlo> = vec![];
        for file in files {
            if let Ok(f) = file {
                let parsed_sqlo: Sqlo = serde_json::from_str(
                    &std::fs::read_to_string(f.path()).sqlo_err(sqlo.ident.span())?,
                )
                .sqlo_err(sqlo.ident.span())?;
                sqlos.push(parsed_sqlo)
            }
        }
        for relation in self.relations.iter() {
            let Relation::ForeignKey(rel_fk) = relation;
            rel_fk.validate(&sqlo, &sqlos)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Relation {
    ForeignKey(RelForeignKey),
}

impl Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Relation::ForeignKey(r) => r.fmt(f),
        }
    }
}

impl TryFrom<PathBuf> for Relation {
    type Error = std::io::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        lazy_static::lazy_static! {
            static ref RE_FK: Regex = Regex::new(r"^fk-(\w+)--(\w+)---(\w+)--(\w+)----(\w+|~)$").unwrap();
        }
        if let Some(filename) = value.file_name() {
            if let Some(filestr) = filename.to_str() {
                if let Some(captures) = RE_FK.captures(filestr) {
                    let res = captures
                        .iter()
                        .skip(1)
                        .filter_map(|f| f)
                        .map(|m| m.as_str())
                        .collect::<Vec<_>>();
                    if res.len() == 4 {
                        return Ok(Relation::ForeignKey(RelForeignKey {
                            from: syn::Ident::new(&res[0], Span::call_site()),
                            field: syn::Ident::new(&res[1], Span::call_site()),
                            to: syn::Ident::new(&res[2], Span::call_site()),
                            ty: syn::parse_quote!(res[3]),
                        }));
                    }
                }
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Sqlo Could not get relation from filename",
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelForeignKey {
    from: syn::Ident,
    field: syn::Ident,
    ty: syn::TypePath,
    to: syn::Ident,
}

impl RelForeignKey {
    fn validate(&self, sqlo: &Sqlo, sqlos: &[Sqlo]) -> Result<(), SqloError> {
        self.validate_existing_fk_struct(sqlo, sqlos)?; // should always be called first (use of unwrap later)
        self.validate_existing_fk_type(sqlo, sqlos)?;
        Ok(())
    }
    fn validate_existing_fk_struct(&self, _sqlo: &Sqlo, sqlos: &[Sqlo]) -> Result<(), SqloError> {
        let matching_sqlo = sqlos.iter().find(|s| s.ident == self.to);
        if matching_sqlo.is_none() {
            return Err(SqloError::new(
                format!("No struct `{}` was  found as derived from Sqlo", &self.to),
                self.to.span(),
            ));
        }
        Ok(())
    }

    fn validate_existing_fk_type(&self, _sqlo: &Sqlo, sqlos: &[Sqlo]) -> Result<(), SqloError> {
        use syn::spanned::Spanned;
        let matching_sqlo = sqlos.iter().find(|s| s.ident == self.to).unwrap(); // safe since validate_fk_struct_called_before

        if matching_sqlo.pk_field.ty.path != self.ty.path {
            return Err(SqloError::new(
                format!(
                    "Field type an foreign key field's type don't match. Expected {} found {}",
                    path_to_string(&matching_sqlo.pk_field.ty.path),
                    path_to_string(&self.ty.path)
                ),
                self.ty.span(),
            ));
        }

        Ok(())
    }
}

impl Display for RelForeignKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = darling::util::path_to_string(&self.ty.path).replace("::", "~~");
        write!(f, fk_pattern!(), self.from, self.field, self.to, ty)
    }
}

fn make_field_relations(field: &Field, sqlo: &Sqlo) -> Vec<Relation> {
    let mut res = vec![];
    if let Some(ref rel) = field.fk {
        res.push(Relation::ForeignKey(RelForeignKey {
            from: sqlo.ident.clone(),
            field: field.ident.clone(),
            ty: field.ty.clone(),
            to: rel.clone(),
        }))
    }
    res
}
