use darling::util::IdentString;
use proc_macro2::{Punct, Spacing};
use syn::Token;

use crate::{
    error::SqloError,
    macros::{ColumnToSql, Context, Fragment, Generator},
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
        // first find if it's a relation
        let relation = match ctx.sqlos.get_relation(&ctx.main_sqlo.ident, &self.base) {
            Ok(rel) => rel,
            Err(_) => {
                // no relation then it could be a sqlo ident
                return match ctx.tables.alias_dot_column(&self.base, &self.member) {
                    Ok(res) => Ok(res.into()),
                    Err(err) => {
                        // left join (=.) can't be value, we buble up field error (if base is sqlo or a related fk)
                        if matches!(self.join, Join::Left) || err.msg().contains("SqloFieldError") {
                            // we track error content because non error variant
                            Err(err)
                        } else {
                            // inner join(.) but base is unknown so we use it as Value
                            let expr: syn::Expr = syn::parse_quote!(#self);
                            Ok(Fragment::from_expr(expr, ctx))
                        }
                    }
                };
            }
        };
        let join = relation.to_join(self.join, ctx)?;
        let column = ctx.tables.alias_dot_column(&self.base, &self.member)?;
        ctx.context.pop();
        Ok((column, join).into())
    }
}

impl syn::parse::Parse for ColExprField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let join = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Join::Left
        } else {
            Join::Inner
        };
        input.parse::<Token![.]>()?;
        let member = input.parse::<syn::Ident>()?;
        Ok(ColExprField::new(ident, member, join))
    }
}
