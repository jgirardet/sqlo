use std::fmt::Display;

use crate::{error::SqloError, relations::Relations, sqlo::Sqlo};

pub struct Sqlos {
    pub(crate) entities: Vec<Sqlo>,
    pub(crate) relations: Relations,
}

impl Sqlos {
    pub fn get<T: Display>(&self, name: T) -> Result<&Sqlo, SqloError> {
        let name = name.to_string();
        self.entities
            .iter()
            .find(|s| s.ident == name)
            .ok_or(SqloError::new_lost(&format!("Can't find entity {}", &name)))
    }
}
