pub mod sqlo_select;
pub mod sqlo_update;
mod wwhere;

pub(crate) struct SqlQuery {
    query: String,
    params: Vec<syn::Expr>,
}
