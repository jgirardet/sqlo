use std::fmt::Display;

use darling::util::IdentString;

use crate::{
    error::SqloError,
    relations::{RelForeignKey, Relation, Relations},
    sqlo::Sqlo,
};

pub struct Sqlos {
    pub(crate) entities: Vec<Sqlo>,
    pub(crate) relations: Relations,
}

impl Sqlos {
    pub fn get<T: AsRef<str> + Display>(&self, name: T) -> Result<&Sqlo, SqloError> {
        self.entities
            .iter()
            .find(|s| s.ident == name.as_ref())
            .ok_or_else(|| SqloError::new_lost(&format!("Can't find Sqlo struct {}", name)))
    }

    pub fn get_by_relation(
        &self,
        to: &IdentString,
        related: &IdentString,
    ) -> Result<&Sqlo, SqloError> {
        let Relation::ForeignKey(relation) = self.relations.find(to, related)?;
        self.get(&relation.from)
    }

    pub fn get_relation(
        &self,
        to: &IdentString,
        related: &IdentString,
    ) -> Result<&RelForeignKey, SqloError> {
        let Relation::ForeignKey(relation) = self.relations.find(to, related)?;
        Ok(relation)
    }
}
