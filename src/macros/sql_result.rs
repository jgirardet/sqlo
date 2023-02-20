use std::collections::BTreeMap;
use std::fmt::Write;

use darling::util::IdentString;
use proc_macro2::TokenStream;
use syn::Expr;

use crate::{
    error::SqloError,
    relations::{RelForeignKey},
    sqlo::Sqlo,
    sqlos::Sqlos,
};

use super::{sqlo_select::SqloSelectParse, wwhere::where_generate_sql};

pub struct SqlResult<'a> {
    main_sqlo: &'a Sqlo,
    relation: Option<&'a RelForeignKey>,
    sqlos: &'a Sqlos,
    columns: String,
    joins: BTreeMap<&'a IdentString, String>,
    wwhere: String,
    arguments: Vec<Expr>,
}

impl<'a> SqlResult<'a> {
    pub fn from_sqlo_parse(
        parsed: SqloSelectParse,
        sqlos: &'a Sqlos,
    ) -> Result<SqlResult, SqloError> {
        let main_sqlo = SqlResult::set_main_and_relation(&parsed, &sqlos)?;
        let mut sqlr = SqlResult::new(main_sqlo, sqlos);
        sqlr.set_relation(&parsed)?;
        sqlr.process_where(&parsed)?;
        sqlr.link_related_in_where(&parsed);
        Ok(sqlr)
    }

    fn new(main_sqlo: &'a Sqlo, sqlos: &'a Sqlos) -> Self {
        SqlResult {
            sqlos,
            main_sqlo,
            relation: Option::default(),
            columns: String::default(),
            wwhere: String::default(),
            arguments: Vec::default(),
            joins: BTreeMap::default(),
        }
    }

    fn set_main_and_relation(
        parsed: &SqloSelectParse,
        sqlos: &'a Sqlos,
    ) -> Result<&'a Sqlo, SqloError> {
        if let Some(ref related) = parsed.related {
            sqlos.get_by_relation(&parsed.entity, related)
        } else {
            sqlos.get(&parsed.entity)
        }
    }
}

impl<'a> SqlResult<'a> {
    fn set_relation(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if let Some(ref related) = parsed.related {
            self.relation = Some(self.sqlos.get_relation(&parsed.entity, related)?);
        }
        Ok(())
    }
    fn process_where(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if let Some(ref wt) = parsed.wwhere {
            let wwhere_sql = where_generate_sql(&self.main_sqlo.ident, &self.sqlos, wt)?;
            self.wwhere = wwhere_sql.query;
            self.arguments.extend(wwhere_sql.params);
        }
        Ok(())
    }

    fn link_related_in_where(&mut self, parsed: &SqloSelectParse) {
        // add fk for relation à the end of where
        if let Some(relation) = &self.relation {
            let prefix = if self.wwhere.is_empty() {
                "WHERE "
            } else {
                " AND "
            };

            write!(
                self.wwhere,
                "{}{}=?",
                prefix,
                relation.get_from_column(&self.sqlos)
            )
            .expect("Error formatting where related  query");
            self.arguments.push(parsed.pk_value.clone().unwrap()); // ok since related exists only if pk_value is parsed.
        }
    }

    // access via related

    fn get_queried_columns(&self) -> String {
        self.main_sqlo.all_columns_as_query()
    }

    fn query(&self) -> String {
        let columns = self.get_queried_columns();
        let tablename = &self.main_sqlo.tablename;
        let where_query = &self.wwhere;
        format!("SELECT DISTINCT {columns} FROM {tablename} {where_query}")
    }

    pub fn expand(&self) -> Result<TokenStream, SqloError> {
        let query = self.query();
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&query);
        }
        let ident = &self.main_sqlo.ident;
        let arguments = &self.arguments;

        Ok(quote::quote! {
            sqlx::query_as!(#ident,#query, #(#arguments),*)
        })
    }
}