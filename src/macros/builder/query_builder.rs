use std::collections::HashSet;
use std::fmt::Write;

use darling::util::IdentString;
use itertools::Itertools;
use syn::Expr;

use super::{Fragment, Generator, Mode, PkValue, QueryParser};

use crate::{
    macros::{Clause, ColExpr, ColumnToSql},
    utils::INSERT_FN_FLAG,
    SqloError,
};

#[derive(Debug, Default, Clone)]
pub struct QueryBuilder {
    subjects: String,
    joins: HashSet<String>,
    wwhere: String,
    group_by: String,
    order_by: String,
    having: String,
    limit: String,
    tablename: String,
    pub arguments: Vec<Expr>,
    pub customs: bool,
}

impl QueryBuilder {
    pub fn extend(&mut self, qr: Fragment) {
        self.arguments.extend(qr.params);
        self.joins.extend(qr.joins);
    }

    fn link_related_entity<T: QueryParser>(&mut self, parsed: &T, ctx: &Generator) {
        match parsed.pk_value() {
            PkValue::Bracketed(pk) => {
                let prefix = if self.wwhere.is_empty() {
                    " WHERE "
                } else {
                    " AND "
                };
                let ident = if let Some(relation) = &ctx.related {
                    relation.get_from_column(ctx.sqlos)
                } else {
                    &ctx.main_sqlo.pk_field.column
                };
                write!(self.wwhere, "{}{}=?", prefix, ident)
                    .expect("Error formatting update query");

                self.arguments.push(pk);
            }
            PkValue::Parenthezide(pk) => {
                let prefix = if self.wwhere.is_empty() {
                    " WHERE "
                } else {
                    " AND "
                };
                let ident = if let Some(relation) = &ctx.related {
                    relation.get_from_column(ctx.sqlos)
                } else {
                    &ctx.main_sqlo.pk_field.column
                };
                write!(self.wwhere, "{}{}=?", prefix, ident)
                    .expect("Error formatting update query");
                if let Expr::Path(p) = pk {
                    let pk_field = &ctx.main_sqlo.pk_field.ident;
                    let with_pk: Expr = syn::parse_quote! {#p.#pk_field};
                    self.arguments.push(with_pk)
                }
            }
            PkValue::None => {} // nothing to be seen here
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
                        &ctx.tables
                            .alias_dot_column(&ctx.main_sqlo.ident, &f.ident)?,
                        &f.ident
                    )
                    .replace('\\', "");
                    res.push(a);
                } else {
                    res.push(
                        ctx.tables
                            .alias_dot_column(&ctx.main_sqlo.ident, &f.ident)?,
                    )
                }
            }
            self.subjects = res.join(", ");
        } else {
            self.customs = true;
            let columns = parsed.columns().iter().fold(
                Ok(Fragment::default()),
                |acc: Result<Fragment, SqloError>, nex| Ok(acc? + nex.column_to_sql(ctx)?),
            )?;
            self.subjects = columns.query.clone();
            self.extend(columns);
        }
        Ok(())
    }

    pub fn set_assigns<T: QueryParser>(
        &mut self,
        parsed: &T,
        ctx: &mut Generator,
    ) -> Result<(), SqloError> {
        let qr = parsed.assigns().column_to_sql(ctx)?;
        self.subjects = qr.query.clone();
        self.extend(qr);
        Ok(())
    }

    pub fn set_values<T: QueryParser>(
        &mut self,
        parsed: &T,
        ctx: &mut Generator,
    ) -> Result<(), SqloError> {
        let assigns = parsed.assigns();
        let mut arguments = Fragment::default();
        let mut columns = vec![];
        for f in &ctx.main_sqlo.fields {
            if let Some(value) = assigns.value(&f.ident) {
                let val = match value {
                    ColExpr::Ident(ident) => {
                        if ident.as_str() == "None" {
                            continue;
                        }
                        value
                    }
                    _ => value,
                };
                arguments = arguments + val.column_to_sql(ctx)?;
                columns.push(f.ident.clone());
            } else if f == &ctx.main_sqlo.pk_field && f.insert_fn.is_some() {
                // use insert_fn if no pk is given
                let ident = syn::Ident::new(INSERT_FN_FLAG, proc_macro2::Span::call_site());
                let arg: syn::Expr = syn::parse_quote! {#ident};
                arguments = arguments + arg.column_to_sql(ctx)?;
                let ident_insert_fn = IdentString::new(ident);
                columns.push(ident_insert_fn);
            }
        }
        let columns = columns.into_iter().join(
            "
            ,",
        );
        self.subjects = format!("({}) VALUES ({})", columns, &arguments.query);
        self.arguments.extend_from_slice(&arguments.params);
        Ok(())
    }

    fn set_tablename(&mut self, ctx: &Generator) -> Result<(), SqloError> {
        self.tablename = match ctx.mode {
            Mode::Select => ctx.tables.tablename_with_alias(&ctx.main_sqlo.ident)?,
            _ => ctx.tables.tablename(&ctx.main_sqlo.ident)?,
        };
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
        self.set_tablename(ctx)?;
        match ctx.mode {
            Mode::Select => self.set_columns(parsed, ctx)?,
            Mode::Update => self.set_assigns(parsed, ctx)?,
            Mode::Insert => {
                self.set_values(parsed, ctx)?;
                return Ok(());
            }
        };
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

        // self.link_related_in_where(parsed, ctx);
        self.link_related_entity(parsed, ctx);
        Ok(())
    }

    pub fn query(&self, ctx: &Generator) -> Result<String, SqloError> {
        let query = match ctx.mode {
            Mode::Select => self.query_select(ctx)?,
            Mode::Update => self.query_update(ctx)?,
            Mode::Insert => self.query_insert(ctx)?,
        };
        Ok(query.trim().into())
    }
    pub fn query_select(&self, ctx: &Generator) -> Result<String, SqloError> {
        let distinct = self.get_distinct(ctx);
        let subjects = &self.subjects;
        let tablename = &self.tablename;
        let joins = self.joins.iter().join(" ");
        let where_query = &self.wwhere;
        let group_by_query = &self.group_by;
        let having_query = &self.having;
        let order_by_query = &self.order_by;
        let limit_query = &self.limit;

        Ok(format!("SELECT{distinct} {subjects} FROM {tablename}{joins}{where_query}{group_by_query}{having_query}{order_by_query}{limit_query}"))
    }

    fn query_update(&self, ctx: &Generator) -> Result<String, SqloError> {
        let subjects = &self.subjects;
        let tablename = &self.tablename;
        let where_query = &self.wwhere;
        let returning_columns = ctx.main_sqlo.to_non_null_columns();

        let returning = if ctx.fetch.is_returning() {
            format!(" RETURNING {}", returning_columns)
        } else {
            "".to_string()
        };

        Ok(format!(
            "UPDATE {tablename} SET {subjects}{where_query}{returning}"
        ))
    }

    fn query_insert(&self, ctx: &Generator) -> Result<String, SqloError> {
        let subjects = &self.subjects;
        let tablename = &self.tablename;
        let returning_columns = ctx.main_sqlo.to_non_null_columns();

        let returning = if ctx.fetch.is_returning() {
            format!(" RETURNING {}", returning_columns)
        } else {
            "".to_string()
        };

        Ok(format!("INSERT INTO {tablename} {subjects}{returning}"))
    }
}
