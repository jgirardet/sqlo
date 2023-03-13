use darling::util::IdentString;
use proc_macro2::Span;
use quote::format_ident;
use regex::Regex;
use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, Type, TypePath};

use std::{
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use crate::{error::SqloError, field::Field, macros::Generator, sqlo::Sqlo, sqlos::Sqlos};

// not const so we can use it in macro
// patter: struct, field, fk_struct, fk_related_name, type
macro_rules! fk_pattern {
    () => {
        "fk-{}--{}---{}--{}----{}"
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relations(Vec<Relation>);

impl FromIterator<Relation> for Relations {
    fn from_iter<T: IntoIterator<Item = Relation>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<Vec<Relation>>())
    }
}

impl Relations {
    /// Build new Relations from a Sqlo instance.
    /// Span is preserved for field
    pub fn from_sqlo(sqlo: &Sqlo) -> Relations {
        sqlo.fields
            .iter()
            .flat_map(|f| make_field_relations(f, sqlo))
            .collect()
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
        Ok(relations
            .into_iter()
            .filter_map(|f| Relation::try_from(f).ok())
            .collect())
    }

    /// Turn relation to a Vec of Relation using thier PathBuf representation.
    pub fn to_files(&self) -> Vec<PathBuf> {
        self.0
            .iter()
            .map(|r| PathBuf::from(r.to_string()))
            .collect()
    }

    pub fn filter_entity(self, mode: &str, ident: &str) -> Relations {
        self.0
            .into_iter()
            .filter(|f| {
                if mode == "from" {
                    f.from == ident
                } else if mode == "to" {
                    f.to == ident
                } else if mode == "both" {
                    f.from == ident || f.to == ident
                } else {
                    unreachable!("Only `to` and `from` are allowed")
                }
            })
            .collect()
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

    pub fn validate(&self, sqlo: &Sqlo, sqlos: &Sqlos) -> Result<(), SqloError> {
        for relation in self.0.iter() {
            let relation = relation;
            relation.validate(sqlo, sqlos)?;
        }
        Ok(())
    }
}

// query impl block
impl Relations {
    pub fn find(&self, to: &IdentString, related: &IdentString) -> Result<&Relation, SqloError> {
        match self.0.iter().find(|r| &r.to == to && &r.related == related) {
            Some(r) => Ok(r),
            None => Err(SqloError::new("No relation found", related.span())),
        }
    }
}

impl TryFrom<PathBuf> for Relation {
    type Error = std::io::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        lazy_static::lazy_static! {
            static ref RE_FK: Regex = Regex::new(r"^fk-(\w+)--(\w+)---(\w+)--(\w+)----([\w~<>]+)$").unwrap();
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
                        return Ok(Relation {
                            from: syn::Ident::new(res[0], Span::call_site()).into(),
                            field: syn::Ident::new(res[1], Span::call_site()).into(),
                            to: syn::Ident::new(res[2], Span::call_site()).into(),
                            related: syn::Ident::new(res[3], Span::call_site()).into(),
                            ty: syn::parse_str(&res[4].replace('~', ":")).map_err(|_| {
                                SqloError::new_lost("Could not parse relation type")
                            })?,
                        });
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
pub struct Relation {
    pub from: IdentString,    // Sqlo struct where fk is defined
    pub field: IdentString,   // in which field
    pub ty: syn::TypePath,    // in which type
    pub to: IdentString,      // Sqlo target struct
    pub related: IdentString, // identifier used by `to` to access `from`
}

impl Relation {
    fn validate(&self, sqlo: &Sqlo, sqlos: &Sqlos) -> Result<(), SqloError> {
        self.validate_existing_fk_struct(sqlo, sqlos)?; // should always be called first (use of unwrap later)
        self.validate_existing_fk_type(sqlo, sqlos)?;
        self.validate_no_field_has_the_same_name_as_related(sqlo, sqlos)?;
        Ok(())
    }
    fn validate_existing_fk_struct(&self, _sqlo: &Sqlo, sqlos: &Sqlos) -> Result<(), SqloError> {
        let _matching_sqlo = sqlos.get(&self.to)?;
        Ok(())
    }

    fn validate_existing_fk_type(&self, _sqlo: &Sqlo, sqlos: &Sqlos) -> Result<(), SqloError> {
        let matching_sqlo = sqlos.get(&self.to)?;

        if !is_the_same_type_or_option(&matching_sqlo.pk_field.ty, &self.ty) {
            return Err(SqloError::new_spanned(
                &self.ty,
                &format!(
                    "Field type an foreign key field's type don't match. Expected {} found {}",
                    format_path(&matching_sqlo.pk_field.ty.path),
                    format_path(&self.ty.path)
                ),
            ));
        }

        Ok(())
    }

    fn validate_no_field_has_the_same_name_as_related(
        &self,
        _sqlo: &Sqlo,
        sqlos: &Sqlos,
    ) -> Result<(), SqloError> {
        let matching_sqlo = sqlos.get(&self.to)?;
        for field in &matching_sqlo.fields {
            if field.ident == self.related {
                return Err(SqloError::new(
                    "related name must be different from all targeted sqlos's fields",
                    self.related.span(),
                ));
            }
        }
        Ok(())
    }

    fn is_self_join(&self) -> bool {
        self.from == self.to
    }

    pub fn to_join(&self, join: Join, ctx: &mut Generator) -> Result<String, SqloError> {
        let to_sqlo = ctx.sqlos.get(&self.to)?;

        ctx.insert_related_alias(self);

        let tablename_plus_alias = ctx.tablename_alias(&self.related)?;
        let lhs;
        let rhs;
        if !self.is_self_join() {
            lhs = ctx.column(&self.related, &self.field)?;
            rhs = ctx.column(&self.to, &to_sqlo.pk_field.ident)?;
        } else {
            rhs = ctx.column(&self.related, &to_sqlo.pk_field.ident)?;
            lhs = ctx.column(&self.to, &self.field)?;
        }

        Ok(format!(
            " {} JOIN {} ON {}={}",
            join, tablename_plus_alias, lhs, rhs,
        ))
    }

    pub fn get_from_column<'a>(&self, sqlos: &'a Sqlos) -> &'a str {
        sqlos
            .get(&self.from)
            .expect("Error: Entity not found from Relation") //should never happen except on first pass
            .field(self.field.as_ident())
            .expect("Sqlo Field not Found, please rebuild")
            .column
            .as_str()
    }
}

impl Display for Relation {
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
        let related = if let Some(related) = &field.related {
            related.clone()
        } else {
            //adjust span if no related is given
            let mut ident = sqlo.ident.as_ident().clone();
            ident.set_span(rel.span());
            as_related_name(&IdentString::new(ident))
        };
        res.push(Relation {
            from: sqlo.ident.clone(),
            field: field.ident.clone(),
            ty: field.ty.clone(),
            to: rel.clone(),
            related,
        })
    }
    res
}

pub fn as_related_name(ident: &IdentString) -> IdentString {
    use heck::ToSnakeCase;
    let name = ident.to_string().to_snake_case();
    format_ident!("{}", syn::Ident::new(&name, ident.span())).into()
}

fn is_the_same_type_or_option(base: &TypePath, target: &TypePath) -> bool {
    if base.path == target.path {
        return true;
    }
    for seg in &target.path.segments {
        if seg.ident == "Option" {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                &seg.arguments
            {
                if let Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.first()
                {
                    return path == &base.path;
                }
            }
        }
    }
    false
}

fn format_path(path: &syn::Path) -> String {
    if let Some(ident) = path.get_ident() {
        return ident.to_string();
    }
    let mut res: Vec<String> = vec![];

    for seg in &path.segments {
        let ident = seg.ident.to_string();
        match &seg.arguments {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                if let Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.first()
                {
                    res.push(format!("{}<{}>", ident, format_path(path)))
                }
            }
            PathArguments::None => res.push(ident),
            _ => unimplemented!("Sqlo: only standard path an option are implemented now"),
        }
    }
    res.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

#[derive(Debug, Clone, Copy)]
pub enum Join {
    Inner,
    Left,
}

impl Display for Join {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inner => write!(f, "INNER"),
            Self::Left => write!(f, "LEFT"),
        }
    }
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

    #[test]
    fn test_try_from_pathbug_for_relation_with_sqare_bracket() {
        let p = PathBuf::from_str(&format!(
            fk_pattern!(),
            "Aaa", "f", "Bbb", "g", "Option<i32>"
        ))
        .unwrap();
        let relation: Relation = p.try_into().unwrap();
        assert_eq!(format_path(&relation.ty.path), "Option<i32>");
    }
}
