use std::io::{self};
use std::path::PathBuf;

use crate::error::SqloError;
use crate::relations::Relations;
use crate::sqlo::Sqlo;

/// Handle every Filesystem IO
pub struct VirtualFile {
    sqlo: Sqlo,
    path: PathBuf,
    relation_path: PathBuf,
}

impl VirtualFile {
    pub fn new(sqlo: Sqlo) -> Self {
        let (path, relation_path) = VirtualFile::get_table_path(&sqlo.ident.to_string());
        Self {
            path,
            sqlo,
            relation_path,
        }
    }

    fn get_table_path(name: &str) -> (PathBuf, PathBuf) {
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
        path.push(name);

        (path, rel_path)
    }

    fn delete_old_relations_files(&self, old: &[PathBuf]) -> io::Result<()> {
        for o in old.iter() {
            std::fs::remove_file(&self.relation_path.join(o))?
        }
        Ok(())
    }

    fn write_fresh_relations_files(&self, fresh: &[PathBuf]) -> io::Result<()> {
        for r in fresh {
            std::fs::write(&self.relation_path.join(&r), &[])?
        }
        Ok(())
    }

    fn write_entity(&self) -> io::Result<()> {
        let content = serde_json::to_vec_pretty(&self.sqlo)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Update informations about Entities and relations in .sqlo dir
    pub fn update(&self) -> Result<(), SqloError> {
        self.write_entity()?;

        let fresh_relations = Relations::from_sqlo(&self.sqlo);
        let existing_relations = Relations::from_path(&self.relation_path)?
            .filter_entity("from", &self.sqlo.ident.to_string());
        self.delete_old_relations_files(
            &fresh_relations.difference_of_other(&existing_relations.to_files()),
        )?;
        self.write_fresh_relations_files(&fresh_relations.to_files())?;
        Ok(())
    }

    /// Validate relations
    pub fn validate(&self) -> Result<(), SqloError> {
        let fresh_relations = Relations::from_sqlo(&self.sqlo);
        fresh_relations.validate(
            &self.sqlo,
            &self.path.parent().expect("Can't find .sqlo/entities dir"),
        )
    }
}
