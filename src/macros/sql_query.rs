use std::collections::HashSet;

use syn::{Expr};

#[derive(Debug)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<syn::Expr>,
    pub joins: HashSet<String>,
}

// Take  a string query
impl From<String> for SqlQuery {
    fn from(s: String) -> Self {
        SqlQuery {
            query: s,
            params: vec![],
            joins: HashSet::default(),
        }
    }
}

// Take to tupple string (query, join)
impl From<(String, String)> for SqlQuery {
    fn from(s: (String, String)) -> Self {
        let mut h = HashSet::default();
        h.insert(s.1);
        SqlQuery {
            query: s.0,
            params: vec![],
            joins: h,
        }
    }
}

// take an Expr so its a argument
impl From<Expr> for SqlQuery {
    fn from(expr: Expr) -> Self {
        SqlQuery {
            query: "?".to_string(),
            params: vec![expr],
            joins: HashSet::default(),
        }
    }
}
