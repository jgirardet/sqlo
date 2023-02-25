use darling::util::IdentString;
use proc_macro2::{Punct, Spacing};

use crate::{sqlo::Sqlo, sqlos::Sqlos, error::SqloError, macros::SqlQuery};

use super::ColumnToSql;

#[derive(Debug)]
pub struct ColExprField {
    base: IdentString,
    member: IdentString,
}

impl From<(syn::Ident, syn::Ident)> for ColExprField {
    fn from(i: (syn::Ident, syn::Ident)) -> Self {
        Self {
            base: i.0.into(),
            member: i.1.into(),
        }
    }
}

impl quote::ToTokens for ColExprField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;
        self.base.to_tokens(tokens);
        tokens.append(Punct::new('.', Spacing::Joint));
        self.member.to_tokens(tokens);
    }
}

impl ColumnToSql for ColExprField {
    fn column_to_sql(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> Result<SqlQuery, SqloError> {
        let relation = sqlos.get_relation(&main_sqlo.ident, &self.base)?;
        let related_sqlo = sqlos.get(&relation.from)?;
        let join = relation.to_inner_join(&sqlos);
        let column = related_sqlo.column(&self.member.as_ident())?;
        return Ok((column, join).into());
    }
}
