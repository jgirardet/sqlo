use crate::{error::SqloError, sqlo::Sqlo, sqlos::Sqlos};

use super::wwhere::{tokenizer::WhereTokenizer, where_generate_sql};

#[derive(Debug)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<syn::Expr>,
}

impl SqlQuery {
    pub fn try_from_option_where_tokenizer(
        wwhere: Option<WhereTokenizer>,
        sqlos: &Sqlos,
        main_sqlo: &Sqlo,
    ) -> Result<Self, SqloError> {
        if let Some(ref wt) = wwhere {
            where_generate_sql(
                &main_sqlo.ident.to_string(), //&self.entity.to_string(),
                sqlos,
                wt,
            )
        } else {
            Ok(SqlQuery {
                query: String::new(),
                params: vec![],
            })
        }
    }
}
