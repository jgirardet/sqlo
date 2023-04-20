use std::collections::HashMap;

use darling::util::IdentString;

use crate::{error::SqloError, relations::Relation, sqlos::Sqlos};

#[derive(Debug, Clone)]
// sqlo_or_related_ident:(char alias, Sqlo ident)
pub struct TableAliases<'a> {
    tables: HashMap<IdentString, (char, IdentString)>,
    sqlos: &'a Sqlos,
}
impl<'a> TableAliases<'a> {
    pub fn new(sqlos: &'a Sqlos) -> Self {
        Self {
            tables: HashMap::default(),
            sqlos,
        }
    }

    pub fn contains(&self, sqlo_or_related: &IdentString) -> bool {
        self.tables.contains_key(sqlo_or_related)
    }

    pub fn get(&self, sqlo_or_related: &IdentString) -> Result<(&char, &IdentString), SqloError> {
        if let Some((ref c, ref ident)) = self.tables.get(sqlo_or_related) {
            Ok((c, ident))
        } else {
            Err(SqloError::new_spanned(
                sqlo_or_related,
                "Sqlo: Invalid alias or identifier",
            ))
        }
    }

    pub fn insert_sqlo(&mut self, sqlo: &IdentString) {
        self.tables
            .insert(sqlo.clone(), (self.get_next_alias(), sqlo.clone()));
    }

    pub fn insert_related(&mut self, rel: &Relation) {
        self.tables.insert(
            rel.related.clone(),
            (self.get_next_alias(), rel.from.clone()),
        );
    }
    pub fn insert_related_alias(&mut self, rel: &Relation) {
        if !&self.contains(&rel.related) {
            self.insert_related(rel)
        }
    }

    /// Give the sql column representation with its alias: alias.column
    pub fn alias_dot_column(
        &mut self,
        sqlo_or_related: &IdentString,
        field: &IdentString,
    ) -> Result<String, SqloError> {
        let (c, sqlo_ident) = self.get(sqlo_or_related)?;
        let column_name = self
            .sqlos
            .get(sqlo_ident)?
            .field(field.as_ident())
            .ok_or_else(|| {
                SqloError::new_spanned(
                    field,
                    format!("SqlorFieldError: no field {} in {}", &field, &sqlo_ident),
                )
            })?
            .column
            .to_string();
        Ok(format!("{c}.{column_name}"))
    }

    pub fn column(
        &mut self,
        sqlo_or_related: &IdentString,
        field: &IdentString,
    ) -> Result<String, SqloError> {
        let (_, sqlo_ident) = self.get(sqlo_or_related)?;
        let column_name = self
            .sqlos
            .get(sqlo_ident)?
            .field(field.as_ident())
            .ok_or_else(|| {
                SqloError::new_spanned(
                    field,
                    format!("SqlorFieldError: no field {} in {}", &field, &sqlo_ident),
                )
            })?
            .column
            .to_string();
        Ok(column_name)
    }

    pub fn tablename_with_alias(&self, sqlo_or_related: &IdentString) -> Result<String, SqloError> {
        let (c, sqlo_ident) = self.get(sqlo_or_related)?;
        Ok(format!(
            "{} {}",
            &self.sqlos.get(sqlo_ident).unwrap().tablename,
            c
        ))
    }

    pub fn tablename(&self, sqlo_or_related: &IdentString) -> Result<String, SqloError> {
        let (_, sqlo_ident) = self.get(sqlo_or_related)?;
        Ok(self.sqlos.get(sqlo_ident).unwrap().tablename.to_string())
    }

    fn get_next_alias(&self) -> char {
        if self.tables.is_empty() {
            'a'
        } else {
            let last_alias = self.tables.values().fold('a', |c, (n, _)| c.max(*n));
            (last_alias as u8 + 1u8).into()
        }
    }
}
