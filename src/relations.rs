use darling::util::{path_to_string, IdentString};
use proc_macro2::Span;
use quote::format_ident;
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
    sqlos::Sqlos,
};

// not const so we can use it in macro
// patter: struct, field, fk_struct, fk_related_name, type
macro_rules! fk_pattern {
    () => {
        "fk-{}--{}---{}--{}----{}"
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relations {
    pub(crate) relations: Vec<Relation>,
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
                .flat_map(|f| make_field_relations(f, sqlo))
                .collect(),
        }
    }

    /// get all relations present in a dir
    /// No span is preserved
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let mut relations = vec![];

        // foreignkeys
        let pat = path.join(format!(fk_pattern!(), "*", "*", "*", "*", "*"));
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
            .iter()
            .filter(|x| !list.contains(x))
            .cloned()
            .collect()
    }

    pub fn validate(&self, sqlo: &Sqlo, path: &Path) -> Result<(), SqloError> {
        let files = std::fs::read_dir(path).sqlo_err(sqlo.ident.span())?;
        let mut sqlos: Vec<Sqlo> = vec![];
        for file in files.flatten() {
            let parsed_sqlo: Sqlo = serde_json::from_str(
                &std::fs::read_to_string(file.path()).sqlo_err(sqlo.ident.span())?,
            )
            .sqlo_err(sqlo.ident.span())?;
            sqlos.push(parsed_sqlo)
        }
        for relation in self.relations.iter() {
            let Relation::ForeignKey(rel_fk) = relation;
            rel_fk.validate(sqlo, &sqlos)?;
        }
        Ok(())
    }
}

// query impl block
impl Relations {
    pub fn find(&self, to: &IdentString, related: &IdentString) -> Result<&Relation, SqloError> {
        match self
            .relations
            .iter()
            .find(|Relation::ForeignKey(r)| &r.to == to && &r.related == related)
        {
            Some(r) => Ok(r),
            None => Err(SqloError::new("No relation found", related.span())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            static ref RE_FK: Regex = Regex::new(r"^fk-(\w+)--(\w+)---(\w+)--(\w+)----([\w~]+)$").unwrap();
        }
        if let Some(filename) = value.file_name() {
            if let Some(filestr) = filename.to_str() {
                if let Some(captures) = RE_FK.captures(filestr) {
                    let res = captures
                        .iter()
                        .skip(1)
                        .flatten()
                        .map(|m| m.as_str())
                        .collect::<Vec<_>>();

                    if res.len() == 5 {
                        return Ok(Relation::ForeignKey(RelForeignKey {
                            from: syn::Ident::new(res[0], Span::call_site()).into(),
                            field: syn::Ident::new(res[1], Span::call_site()).into(),
                            to: syn::Ident::new(res[2], Span::call_site()).into(),
                            related: syn::Ident::new(res[3], Span::call_site()).into(),
                            ty: syn::parse_str(&res[4].replace('~', ":")).map_err(|_| {
                                SqloError::new_lost("Could not parse relation type")
                            })?,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelForeignKey {
    pub from: IdentString,
    pub field: IdentString,
    pub ty: syn::TypePath,
    pub to: IdentString,
    pub related: IdentString,
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
                &format!("No struct `{}` was  found as derived from Sqlo", &self.to),
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
                &format!(
                    "Field type an foreign key field's type don't match. Expected {} found {}",
                    path_to_string(&matching_sqlo.pk_field.ty.path),
                    path_to_string(&self.ty.path)
                ),
                self.ty.span(),
            ));
        }

        Ok(())
    }

    pub fn to_inner_join(&self, sqlos: &Sqlos) -> String {
        let from_sqlo = {
            let ref this = sqlos;
            let name = &self.from;
            this.entities
                .iter()
                .find(|s| s.ident == name.as_ref())
                .ok_or_else(|| SqloError::new_lost(&format!("Can't find Sqlo struct {}", name)))
        }
        .expect("Error: Entity not found from Relation"); //should never happen except on first pass
        let to_sqlo = {
            let ref this = sqlos;
            let name = &self.to;
            this.entities
                .iter()
                .find(|s| s.ident == name.as_ref())
                .ok_or_else(|| SqloError::new_lost(&format!("Can't find Sqlo struct {}", name)))
        }
        .expect("Error: Entity not found from Relation"); //should never happen except on first pass
        let from_field = from_sqlo
            .field(&self.field)
            .expect("Sqlo Field not Found, please rebuild");
        format!(
            "INNER JOIN {} ON {}.{}={}.{}",
            &from_sqlo.tablename,
            &to_sqlo.tablename,
            &to_sqlo.pk_field.column,
            &from_sqlo.tablename,
            &from_field.column
        )
    }

    pub fn get_from_column<'a>(&self, sqlos: &'a Sqlos) -> &'a str {
        let from_sqlo = {
            let ref this = sqlos;
            let name = &self.from;
            this.entities
                .iter()
                .find(|s| s.ident == name.as_ref())
                .ok_or_else(|| SqloError::new_lost(&format!("Can't find Sqlo struct {}", name)))
        }
        .expect("Error: Entity not found from Relation"); //should never happen except on first pass
                                                          // let to_sqlo = sqlos
                                                          //     .get(&self.to)
                                                          //     .expect("Error: Entity not found from Relation"); //should never happen except on first pass
        let from_field = from_sqlo
            .field(&self.field)
            .expect("Sqlo Field not Found, please rebuild");
        from_field.column.as_str()
    }
}

impl Display for RelForeignKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = darling::util::path_to_string(&self.ty.path).replace("::", "~~");
        write!(
            f,
            fk_pattern!(),
            self.from, self.field, self.to, self.related, ty
        )
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
            related: field
                .related
                .clone()
                .unwrap_or_else(|| as_related_name(&sqlo.ident)),
        }))
    }
    res
}

pub fn as_related_name(ident: &IdentString) -> IdentString {
    use heck::ToSnakeCase;
    let name = ident.to_string().to_snake_case();
    format_ident!("{}", syn::Ident::new(&name, ident.span())).into()
}

#[cfg(test)]
mod test_relations {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_try_from_pathbug_for_relation() {
        let p = PathBuf::from_str(&format!(fk_pattern!(), "Aaa", "f", "Bbb", "g", "i32")).unwrap();
        let _: Relation = p.try_into().unwrap();
    }

    #[test]
    fn test_try_from_pathbug_for_relation_with_tilde_in_type() {
        let p = PathBuf::from_str(&format!(fk_pattern!(), "Aaa", "f", "Bbb", "g", "path~~i32"))
            .unwrap();
        let _: Relation = p.try_into().unwrap();
    }
}
