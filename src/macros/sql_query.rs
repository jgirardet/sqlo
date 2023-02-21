use std::collections::{HashSet};

use darling::util::IdentString;

#[derive(Debug)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<syn::Expr>,
    pub joins: HashSet<String>,
}
