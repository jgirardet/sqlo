use std::collections::HashSet;

#[derive(Debug)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<syn::Expr>,
    pub joins: HashSet<String>,
}
