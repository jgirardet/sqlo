use std::collections::HashSet;
use std::fmt::Write;

use itertools::Itertools;
use syn::Expr;

use super::{Fragment, Generator, Mode, SqloQueryParse};
use crate::{error::SqloError, macros::ColumnToSql};

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

macro_rules! impl_process_sqlqueries {
    ($($clause:ident)+) => {
        paste::paste! {
            impl QueryBuilder {
                $(
                fn [<process_ $clause>](&mut self, parsed: &SqloQueryParse, ctx: &mut Generator) -> Result<(), SqloError> {
                    if let Some(case) = &parsed.$clause {
                        let qr = case.column_to_sql(ctx)?;
                        self.$clause = qr.query.clone();
                        self.extend(qr);
                    }
                    Ok(())
                }
            )+
            }
        }
    };
}

impl_process_sqlqueries!(wwhere group_by order_by limit having);

impl QueryBuilder {
    pub fn extend(&mut self, qr: Fragment) {
        self.arguments.extend(qr.params);
        self.joins.extend(qr.joins);
    }

    fn link_related_in_where(&mut self, parsed: &SqloQueryParse, ctx: &Generator) {
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
            self.arguments.push(parsed.pk_value.clone().unwrap()); // ok since related exists only if pk_value is parsed.
        }
    }

    pub fn set_columns(
        &mut self,
        parsed: &SqloQueryParse,
        ctx: &mut Generator,
    ) -> Result<(), SqloError> {
        if parsed.customs.is_empty() {
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
            let columns = parsed.customs.iter().fold(
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

    pub fn parse(&mut self, parsed: &SqloQueryParse, ctx: &mut Generator) -> Result<(), SqloError> {
        self.set_columns(parsed, ctx)?;
        self.process_wwhere(parsed, ctx)?;
        self.link_related_in_where(parsed, ctx);
        self.process_group_by(parsed, ctx)?;
        self.process_having(parsed, ctx)?;
        self.process_order_by(parsed, ctx)?;
        self.process_limit(parsed, ctx)?;
        // self.set_custom_struct(parsed, ctx);
        Ok(())
    }

    pub fn query(&self, ctx: &Generator) -> Result<String, SqloError> {
        let distinct = self.get_distinct(ctx);
        let columns = &self.columns;
        let tablename = ctx
            // .table_aliases
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
