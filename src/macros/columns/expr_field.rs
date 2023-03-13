use darling::util::IdentString;
use proc_macro2::{Punct, Spacing};

use crate::{
    error::SqloError,
    macros::{Context, Fragment, Generator, ColumnToSql},
    relations::Join,
};


#[derive(Debug, Clone)]
pub struct ColExprField {
    base: IdentString,
    member: IdentString,
    join: Join,
}

impl ColExprField {
    pub fn new(base: syn::Ident, member: syn::Ident, join: Join) -> Self {
        Self {
            base: base.into(),
            member: member.into(),
            join,
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
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError> {
        ctx.context.push(Context::Field);
        let relation = match ctx.sqlos.get_relation(&ctx.main_sqlo.ident, &self.base) {
            Ok(rel) => rel,
            Err(_) => {
                return Ok(ctx.column(&self.base, &self.member)?.into());
            }
        };
        let join = relation.to_join(self.join, ctx)?;
        let column = ctx.column(&self.base, &self.member)?;
        ctx.context.pop();
        Ok((column, join).into())
    }
}
