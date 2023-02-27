use std::collections::HashSet;
use std::fmt::Write;

use darling::util::IdentString;
use itertools::Itertools;
use proc_macro2::TokenStream;
use syn::Expr;

use crate::{error::SqloError, relations::RelForeignKey, sqlo::Sqlo, sqlos::Sqlos};

use super::{sqlo_select::SqloSelectParse, wwhere::process_where, ColumnToSql};

pub struct SqlResult<'a> {
    main_sqlo: &'a Sqlo,
    columns: String,
    relation: Option<&'a RelForeignKey>,
    sqlos: &'a Sqlos,
    joins: HashSet<String>,
    wwhere: String,
    arguments: Vec<Expr>,
    customs: bool,
    custom_struct: Option<IdentString>,
}

impl<'a> SqlResult<'a> {
    pub fn from_sqlo_parse(
        parsed: SqloSelectParse,
        sqlos: &'a Sqlos,
    ) -> Result<SqlResult, SqloError> {
        let main_sqlo = SqlResult::set_main_and_relation(&parsed, &sqlos)?;
        let mut sqlr = SqlResult::new(main_sqlo, sqlos);
        sqlr.set_columns(&parsed)?;
        sqlr.set_relation(&parsed)?;
        sqlr.process_where(&parsed)?;
        sqlr.link_related_in_where(&parsed);
        sqlr.set_custom_struct(&parsed);
        Ok(sqlr)
    }

    fn new(main_sqlo: &'a Sqlo, sqlos: &'a Sqlos) -> Self {
        SqlResult {
            sqlos,
            main_sqlo,
            columns: String::default(),
            relation: Option::default(),
            wwhere: String::default(),
            arguments: Vec::default(),
            joins: HashSet::default(),
            customs: false,
            custom_struct: None,
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

    fn set_custom_struct(&mut self, parsed: &SqloSelectParse) {
        self.custom_struct = parsed.custom_struct.clone();
    }

    fn process_where(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if let Some(ref wt) = parsed.wwhere {
            let wwhere_sql = process_where(&self.main_sqlo.ident, &self.sqlos, wt)?;
            self.wwhere = wwhere_sql.query;
            self.arguments.extend(wwhere_sql.params);
            self.joins.extend(wwhere_sql.joins);
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

    fn set_columns(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if parsed.customs.is_empty() {
            self.columns = self.main_sqlo.all_columns_as_query.to_string();
        } else {
            self.customs = true;
            let mut res = vec![];
            for col in &parsed.customs {
                let query_column = col.column_to_sql(&self.main_sqlo, &self.sqlos)?;
                res.push(query_column.query);
                self.joins.extend(query_column.joins);
                self.arguments.extend(query_column.params);
            }
            self.columns = res.join(", ");
        }
        Ok(())
    }

    fn query(&self) -> String {
        let columns = &self.columns;
        let tablename = &self.main_sqlo.tablename;
        let joins = self.joins.iter().join(" ");
        let where_query = &self.wwhere;
        format!("SELECT DISTINCT {columns} FROM {tablename} {joins} {where_query}")
            .trim_end()
            .into()
    }

    pub fn expand(&self) -> Result<TokenStream, SqloError> {
        let query = self.query();
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&query);
        }
        let ident = if let Some(ident) = &self.custom_struct {
            ident
        } else {
            &self.main_sqlo.ident
        };
        let arguments = &self.arguments;

        if self.customs && self.custom_struct.is_none() {
            Ok(quote::quote! {
                sqlx::query!(#query, #(#arguments),*)
            })
        } else {
            Ok(quote::quote! {
                sqlx::query_as!(#ident,#query, #(#arguments),*)
            })
        }
    }
}
