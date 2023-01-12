use proc_macro2::TokenStream;
use syn::Expr;

use crate::sqlos::Sqlos;

use super::{FromContext, SelectContext};
use std::fmt::Display;

use quote::quote;

pub trait Validate {
    #[allow(unused_variables)]
    fn validate(&self, sqlos: &Sqlos) -> syn::Result<()> {
        Ok(())
    }
}

#[allow(unused_variables)]
pub trait Sqlize {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        unimplemented!("Not implemented in context select")
    }
    fn ffrom(&self, acc: &mut Sqlized, context: &FromContext) -> syn::Result<()> {
        unimplemented!("Not implemented in context from")
    }
    // fn wwhere(&self, sqlized: &mut Sqlized) {}
}

#[derive(Debug, Default)]
pub struct Sqlized {
    sql: Vec<String>,
    params: Vec<Expr>,
}

impl Sqlized {
    pub fn new(sql: Vec<String>, params: Vec<Expr>) -> Self {
        Self { sql, params }
    }

    pub fn sql(&self) -> &Vec<String> {
        &self.sql
    }

    pub fn params(&self) -> &Vec<Expr> {
        &self.params
    }

    // Append only some sql
    pub fn append_sql(&mut self, value: String) {
        self.sql.push(value);
    }

    // concat qroup's sql before appending
    pub fn append_group(&mut self, group: Sqlized) {
        self.sql.push(group.sql.join(""));
        self.params.extend(group.params.into_iter());
    }

    // concat qroup's sql before appending
    pub fn append_group_with(&mut self, group: Sqlized, sep: &str) {
        self.sql.push(group.sql.join(sep));
        self.params.extend(group.params.into_iter());
    }

    // Append sql and params
    pub fn append(&mut self, other: Sqlized) {
        self.sql.extend(other.sql.into_iter());
        self.params.extend(other.params.into_iter());
    }

    pub fn expand(self) -> TokenStream {
        let sql = self.to_string();
        let params = self.params;

        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&sql);
        }
        quote! {
            sqlx::query![#sql,#(#params),*]
        }
    }
}

impl Display for Sqlized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sql.join(" "))
    }
}
