#[derive(Debug, PartialEq, Eq)]
pub enum Context {
    Call,
    Cast,
    SubQuery,
    Where,
    Field,
    OrderBy,
    Operation,
    Paren,
    Unary,
    Assign,
}
