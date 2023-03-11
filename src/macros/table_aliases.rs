use std::collections::HashMap;

use darling::util::IdentString;

use crate::{error::SqloError, relations::RelForeignKey, sqlos::Sqlos};

#[derive(Debug, Default, Clone)]
// sqlo_or_related_ident:(char alias, Sqlo ident)
pub struct TableAliases(HashMap<IdentString, (char, IdentString)>);
impl TableAliases {
    pub fn insert_sqlo(&mut self, sqlo: &IdentString) {
        self.0
            .insert(sqlo.clone(), (self.get_next_alias(), sqlo.clone()));
    }

    pub fn insert_related(&mut self, rel: &RelForeignKey) {
        self.0.insert(
            rel.related.clone(),
            (self.get_next_alias(), rel.from.clone()),
        );
    }

    pub fn column(
        &mut self,
        sqlo_or_related: &IdentString,
        field: &IdentString,
        sqlos: &Sqlos,
    ) -> Result<String, SqloError> {
        if let Some((c, sqlo_ident)) = self.0.get(sqlo_or_related) {
            let column_name = sqlos
                .get(sqlo_ident)?
                .field(field.as_ident())
                .ok_or_else(|| {
                    SqloError::new_spanned(field, format!("No field {} in {}", &field, &sqlo_ident))
                })?
                .column
                .to_string();
            Ok(format!("{c}.{column_name}"))
        } else {
            Err(SqloError::new_spanned(sqlo_or_related, "Not Found"))
        }
    }

    pub fn contains(&self, sqlo_or_related: &IdentString) -> bool {
        self.0.contains_key(sqlo_or_related)
    }

    pub fn tablename(
        &self,
        sqlo_or_related: &IdentString,
        sqlos: &Sqlos,
    ) -> Result<String, SqloError> {
        if let Some((c, sqlo_ident)) = self.0.get(sqlo_or_related) {
            Ok(format!(
                "{} {}",
                &sqlos.get(sqlo_ident).unwrap().tablename,
                c
            ))
        } else {
            Err(SqloError::new_spanned(sqlo_or_related, "Not Found"))
        }
    }

    fn get_next_alias(&self) -> char {
        if self.0.is_empty() {
            'a'
        } else {
            let last_alias = self.0.values().fold('a', |c, (n, _)| c.max(*n));
            (last_alias as u8 + 1u8).into()
        }
    }
}
