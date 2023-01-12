#[derive(Debug, Clone)]
/// Are we operating with query or query as ?
pub enum QueryContext {
    Sqlo(QueryMoment),
    SqloAs(QueryMoment),
}

///  Querymoment give a hint about the context where the sqltoken has been used:
///     InPhrase: Starting the whole Phrase
///     InClause: Each SqlToken is precessed at the root of the clause
///     InExpr: The SqlToken is processed inside another one like `id` in CALL(id), id + id, ...
#[derive(Debug, Clone)]
pub enum QueryMoment {
    InPhrase,
    InClause,
    InExpr,
}

impl QueryContext {
    pub fn lower(&mut self) -> QueryContext {
        match self {
            QueryContext::Sqlo(QueryMoment::InPhrase) => QueryContext::Sqlo(QueryMoment::InClause),
            QueryContext::SqloAs(QueryMoment::InPhrase) => {
                QueryContext::SqloAs(QueryMoment::InClause)
            }
            QueryContext::SqloAs(QueryMoment::InClause) => {
                QueryContext::SqloAs(QueryMoment::InExpr)
            }
            QueryContext::Sqlo(QueryMoment::InClause) => QueryContext::Sqlo(QueryMoment::InExpr),

            QueryContext::SqloAs(QueryMoment::InExpr) => QueryContext::SqloAs(QueryMoment::InExpr),
            QueryContext::Sqlo(QueryMoment::InExpr) => QueryContext::Sqlo(QueryMoment::InExpr),
        }
    }
}
