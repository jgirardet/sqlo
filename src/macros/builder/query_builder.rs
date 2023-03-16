use std::collections::HashSet;
use std::fmt::Write;

use itertools::Itertools;
use syn::Expr;

use super::{Fragment, Generator, Mode, QueryParser};
use crate::{
    macros::{Clause, ColumnToSql},
    SqloError,
};

#[derive(Debug, Default, Clone)]
pub struct QueryBuilder {
    columns: String,
    joins: HashSet<String>,
    wwhere: String,
    group_by: String,
    order_by: String,
    having: String,
    limit: String,
    pub arguments: Vec<Expr>,
    pub customs: bool,
}

impl QueryBuilder {
    pub fn extend(&mut self, qr: Fragment) {
        self.arguments.extend(qr.params);
        self.joins.extend(qr.joins);
    }

    fn link_related_in_where<T: QueryParser>(&mut self, parsed: &T, ctx: &Generator) {
        // add fk for relation à the end of where
        if let Some(relation) = &ctx.related {
            let prefix = if self.wwhere.is_empty() {
                " WHERE "
            } else {
                " AND "
            };

            write!(
                self.wwhere,
                "{}{}=?",
                prefix,
                relation.get_from_column(ctx.sqlos)
            )
            .expect("Error formatting where related  query");
            self.arguments.push(parsed.pk_value().clone().unwrap()); // ok since related exists only if pk_value is parsed.
        }
    }

    pub fn set_columns<T: QueryParser>(
        &mut self,
        parsed: &T,
        ctx: &mut Generator,
    ) -> Result<(), SqloError> {
        if parsed.columns().is_empty() {
            let mut res = vec![];
            for f in ctx.main_sqlo.fields.iter() {
                // we write full query if name or type isn't the same between rust struct and database
                if f.type_override || f.ident != f.column || f.ident == "id" {
                    let a = format!(
                        r#"{} as "{}:_""#,
                        &ctx.column(&ctx.main_sqlo.ident, &f.ident)?,
                        &f.ident
                    )
                    .replace('\\', "");
                    res.push(a);
                } else {
                    res.push(ctx.column(&ctx.main_sqlo.ident, &f.ident)?)
                }
            }
            self.columns = res.join(", ");
        } else {
            self.customs = true;
            let columns = parsed.columns().iter().fold(
                Ok(Fragment::default()),
                |acc: Result<Fragment, SqloError>, nex| Ok(acc? + nex.column_to_sql(ctx)?),
            )?;
            self.columns = columns.query.clone();
            self.extend(columns);
        }
        Ok(())
    }

    fn get_distinct(&self, ctx: &Generator) -> &str {
        // auto add distinct
        // non need of distinct for plain sqlo struct query if no join.
        // but necessary for everything else
        if self.customs || ctx.custom_struct.is_some() || !self.joins.is_empty() {
            " DISTINCT"
        } else {
            ""
        }
    }

    pub fn parse<T: QueryParser>(
        &mut self,
        parsed: &T,
        ctx: &mut Generator,
    ) -> Result<(), SqloError> {
        self.set_columns(parsed, ctx)?;
        for clause in parsed.clauses().iter() {
            match clause {
                // order matters
                Clause::Where(x) => {
                    let qr = x.column_to_sql(ctx)?;
                    self.wwhere = qr.query.clone();
                    self.extend(qr);
                }
                Clause::GroupBy(x) => {
                    let qr = x.column_to_sql(ctx)?;
                    self.group_by = qr.query.clone();
                    self.extend(qr);
                }
                Clause::Having(x) => {
                    let qr = x.column_to_sql(ctx)?;
                    self.having = qr.query.clone();
                    self.extend(qr);
                }
                Clause::OrderBy(x) => {
                    let qr = x.column_to_sql(ctx)?;
                    self.order_by = qr.query.clone();
                    self.extend(qr);
                }
                Clause::Limit(x) => {
                    let qr = x.column_to_sql(ctx)?;
                    self.limit = qr.query.clone();
                    self.extend(qr);
                }
            }
        }
        self.link_related_in_where(parsed, ctx);
        Ok(())
    }

    pub fn query(&self, ctx: &Generator) -> Result<String, SqloError> {
        let distinct = self.get_distinct(ctx);
        let columns = &self.columns;
        let tablename = ctx
            .tablename_alias(&ctx.main_sqlo.ident)?;
        let joins = self.joins.iter().join(" ");
        let where_query = &self.wwhere;
        let group_by_query = &self.group_by;
        let having_query = &self.having;
        let order_by_query = &self.order_by;
        let limit_query = &self.limit;
        if let Mode::Select = ctx.mode {
            Ok(format!("SELECT{distinct} {columns} FROM {tablename}{joins}{where_query}{group_by_query}{having_query}{order_by_query}{limit_query}")
            .trim_end()
        .into())
        } else {
            Err(SqloError::new_lost("Query Not supported"))
        }
    }
}
