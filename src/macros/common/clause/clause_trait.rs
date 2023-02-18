use crate::macros::common::SqlTokens;

pub trait ClauseTrait {
    fn sqltokens(&self) -> SqlTokens;
}
