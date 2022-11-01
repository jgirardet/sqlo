use std::io::{self};
use std::path::PathBuf;

use crate::error::SqloError;
use crate::relations::Relations;
use crate::sqlo::Sqlo;
use crate::sqlos::Sqlos;

/// Handle every Filesystem IO
pub struct VirtualFile {
    path: PathBuf,
    relation_path: PathBuf,
}

impl VirtualFile {
    pub fn new() -> Self {
        let (path, relation_path) = VirtualFile::get_table_path();
        Self {
            path,
            relation_path,
        }
    }

    fn get_table_path() -> (PathBuf, PathBuf) {
        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push(".sqlo/");
        if !path.exists() {
            std::fs::create_dir(&path).expect("Unable to create .sqlo/");
        }
        let mut rel_path = path.clone();
        path.push("entities/");
        rel_path.push("relations/");
        if !path.exists() {
            std::fs::create_dir(&path).expect("Unable to create .sqlo/entities dir");
        }
        if !rel_path.exists() {
            std::fs::create_dir(&rel_path).expect("Unable to create .sqlo/relations dir");
        }

        (path, rel_path)
    }

    fn delete_old_relations_files(&self, old: &[PathBuf]) -> io::Result<()> {
        for o in old.iter() {
            std::fs::remove_file(&self.relation_path.join(o))?
        }
        Ok(())
    }

    /// Loads all content of .sqlo in an Sqlos
    pub fn load(&self) -> Result<Sqlos, SqloError> {
        let relations = Relations::from_path(&self.relation_path)?;
        let mut entities = vec![];
        for f in std::fs::read_dir(&self.path)? {
            if let Ok(file) = f {
                let sqlo = serde_json::from_str(&std::fs::read_to_string(file.path())?)?;
                entities.push(sqlo);
            }
        }

        Ok(Sqlos {
            relations,
            entities,
        })
    }

    fn write_fresh_relations_files(&self, fresh: &[PathBuf]) -> io::Result<()> {
        for r in fresh {
            std::fs::write(&self.relation_path.join(&r), &[])?
        }
        Ok(())
    }

    fn write_entity(&self, sqlo: &Sqlo) -> io::Result<()> {
        let content = serde_json::to_vec_pretty(&sqlo)?;
        std::fs::write(&self.path.join(&sqlo.ident.to_string()), content)?;
        Ok(())
    }

    /// Update informations about Entities and relations in .sqlo dir
    pub fn update(&self, current_sqlo: &Sqlo) -> Result<(), SqloError> {
        self.write_entity(current_sqlo)?;

        let fresh_relations = Relations::from_sqlo(&current_sqlo);
        let existing_relations = Relations::from_path(&self.relation_path)?
            .filter_entity("from", &current_sqlo.ident.to_string());
        self.delete_old_relations_files(
            &fresh_relations.difference_of_other(&existing_relations.to_files()),
        )?;
        self.write_fresh_relations_files(&fresh_relations.to_files())?;
        Ok(())
    }

    /// Validate relations
    pub fn validate(&self, sqlo: &Sqlo) -> Result<(), SqloError> {
        let fresh_relations = Relations::from_sqlo(&sqlo);
        fresh_relations.validate(&sqlo, &self.path)
    }
}
