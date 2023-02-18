use crate::macros::common::{
    AliasSqlo,  QueryContext, QueryMoment, SelectContext, Sqlize, Sqlized,
};
use syn::{Expr, Member};

use crate::macros::common::Validate;

use super::token_ident::TokenIdent;

#[derive(Debug)]
pub struct TokenField {
    base: TokenIdent,
    member: TokenIdent,
}

impl_trait_to_tokens_for_tokens!(TokenField, base, member);

impl TryFrom<Expr> for TokenField {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        if let Expr::Field(ref exprfield) = expr {
            if let Expr::Path(p) = exprfield.base.as_ref() {
                let base: TokenIdent = p.try_into()?;
                if let Member::Named(ref i) = exprfield.member {
                    return Ok(TokenField {
                        base,
                        member: i.into(),
                    });
                }
            }
        }
        return_error!(
            expr,
            "Invalid expression: should be sqlo_struct.fieldname or alias.fieldname"
        )
    }
}

impl Validate for TokenField {}

impl Sqlize for TokenField {
    fn sselect(&self, acc: &mut Sqlized, context: &mut SelectContext) -> syn::Result<()> {
        let mut group = Sqlized::default();
        // let mut base = String::new();
        // for sqlo_alias in context.alias_sqlos.iter() {
        //     if let Some(alias) = sqlo_alias.alias {
        //         if alias == &self.base {
        //             base = alias.to_string();
        //             break;
        //         }
        //     }
        //     if sqlo_alias.sqlo.ident == self.base {
        //         base = sqlo_alias.sqlo.tablename.to_string();
        //         break;
        //     }
        // }
        let base = match context.alias_sqlos.find(&self.base) {
            Ok(AliasSqlo::Alias(s, a)) => a.to_string(),
            Ok(AliasSqlo::Ident(s)) => s.tablename.to_string(),
            _ => "".to_string(),
        };

        if !base.is_empty() {
            group.append_sql(base);
            group.append_sql(".".to_string());
            self.member.sselect(&mut group, context)?;
            // we force a cast to make the result field name not be column name, but sqlo field name.
            // It applies only in the InClause context.
            match context.query_context {
                QueryContext::Sqlo(QueryMoment::InClause)
                | QueryContext::SqloAs(QueryMoment::InClause) => {
                    group.append_sql(format!(" AS {}", self.member.to_string()));
                }
                _ => {}
            }
            acc.append_group(group);
            return Ok(());
        }
        return_error!(&self.base, "No Sqlo struct or alias found in FROM clause")
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenField {
    fn stry(&self) -> String {
        format!("{}.{}", self.base.stry(), self.member.stry())
    }
}
