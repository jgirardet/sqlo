use std::fmt::Display;

use crate::{error::SqloError, relations::Relations, sqlo::Sqlo};

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
}
