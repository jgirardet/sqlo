use std::fmt::Write;

use std::collections::HashSet;

use darling::util::IdentString;
use itertools::Itertools;
use proc_macro2::TokenStream;
use syn::Expr;

use crate::{error::SqloError, relations::RelForeignKey, sqlo::Sqlo, sqlos::Sqlos};

use super::{sqlo_select::SqloSelectParse, wwhere::process_where, ColumnToSql, SqlQuery};

pub struct SqlResult<'a> {
    pub main_sqlo: &'a Sqlo,
    pub sqlos: &'a Sqlos,
    pub alias: HashSet<IdentString>,
    columns: String,
    joins: HashSet<String>,
    wwhere: String,
    order_by: String,
    arguments: Vec<Expr>,
    relation: Option<&'a RelForeignKey>,
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
        sqlr.process_order_by(&parsed)?;
        sqlr.set_custom_struct(&parsed);
        Ok(sqlr)
    }

    fn new(main_sqlo: &'a Sqlo, sqlos: &'a Sqlos) -> Self {
        SqlResult {
            sqlos,
            main_sqlo,
            alias: HashSet::default(),
            columns: String::default(),
            relation: Option::default(),
            wwhere: String::default(),
            arguments: Vec::default(),
            joins: HashSet::default(),
            customs: false,
            custom_struct: None,
            order_by: String::default(),
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
    fn extend(&mut self, qr: SqlQuery) {
        self.arguments.extend(qr.params);
        self.joins.extend(qr.joins);
    }

    fn set_relation(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if let Some(ref related) = parsed.related {
            self.relation = Some(self.sqlos.get_relation(&parsed.entity, related)?);
        }
        Ok(())
    }

    fn set_custom_struct(&mut self, parsed: &SqloSelectParse) {
        self.custom_struct = parsed.custom_struct.clone();
    }

    fn process_order_by(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if let Some(order_bys) = &parsed.order_by {
            let qr = order_bys.column_to_sql(self)?;
            self.order_by = qr.query.clone();
            self.extend(qr);
        }
        Ok(())
    }

    fn process_where(&mut self, parsed: &SqloSelectParse) -> Result<(), SqloError> {
        if let Some(ref wt) = parsed.wwhere {
            let wwhere_sql = process_where(&self.main_sqlo.ident, &self.sqlos, wt)?;
            self.wwhere = wwhere_sql.query.clone();
            self.extend(wwhere_sql);
        }
        Ok(())
    }

    fn link_related_in_where(&mut self, parsed: &SqloSelectParse) {
        // add fk for relation à the end of where
        if let Some(relation) = &self.relation {
            let prefix = if self.wwhere.is_empty() {
                " WHERE "
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
            let columns = parsed.customs.iter().fold(
                Ok(SqlQuery::default()),
                |acc: Result<SqlQuery, SqloError>, nex| Ok(acc? + nex.column_to_sql(self)?),
            )?;
            self.columns = columns.query.clone();
            self.extend(columns);
        }
        Ok(())
    }

    fn query(&self) -> String {
        let columns = &self.columns;
        let tablename = &self.main_sqlo.tablename;
        let joins = self.joins.iter().join(" ");
        let where_query = &self.wwhere;
        let order_by_query = &self.order_by;
        format!("SELECT DISTINCT {columns} FROM {tablename}{joins}{where_query}{order_by_query}")
            .trim_end()
            .into()
    }

    pub fn expand(&self) -> Result<TokenStream, SqloError> {
        let query = self.query();
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            println!("query: {}", &query);
        } else if std::env::var("SQLO_DEBUG_QUERY_ALL").is_ok() {
            let dd = format!("query: {} \n args: {:?}", &query, &self.arguments);
            println!("{}", dd);
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
