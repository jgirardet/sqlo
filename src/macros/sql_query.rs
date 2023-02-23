use std::collections::HashSet;

#[derive(Debug)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<syn::Expr>,
    pub joins: HashSet<String>,
}

impl From<String> for SqlQuery {
    fn from(s: String) -> Self {
        SqlQuery {
            query: s,
            params: vec![],
            joins: HashSet::default(),
        }
    }
}
