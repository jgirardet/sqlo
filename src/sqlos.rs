use std::{fmt::Display, path::PathBuf};

use crate::{error::SqloError, relations::Relations, sqlo::Sqlo};

pub(crate) struct Sqlos {
    pub(crate) entities: Vec<Sqlo>,
    pub(crate) relations: Relations,
}

impl Sqlos {
    /// Loads all content of .sqlo in an Sqlos
    pub fn load() -> Result<Self, SqloError> {
        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push(".sqlo/");
        let mut rel_path = path.clone();
        rel_path.push("relations");
        path.push("entities");
        let relations = Relations::from_path(&rel_path)?;
        let mut entities = vec![];
        for f in std::fs::read_dir(&path)? {
            if let Ok(file) = f {
                let sqlo = serde_json::from_str(&std::fs::read_to_string(file.path())?)?;
                entities.push(sqlo);
            }
        }

        Ok(Self {
            relations,
            entities,
        })
    }

    pub fn get<T: Display>(&self, name: T) -> Result<&Sqlo, SqloError> {
        let name = name.to_string();
        self.entities
            .iter()
            .find(|s| s.ident == name)
            .ok_or(SqloError::new_lost(&format!("Can't find entity {}", &name)))
    }
}
