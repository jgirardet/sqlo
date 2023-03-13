use darling::util::IdentString;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{punctuated::Punctuated, Token};

use crate::macros::ColumnToSql;

use super::ColExpr;


#[derive(Debug, Clone)]
pub struct ColExprCase {
    case: Option<Box<ColExpr>>,
    arms: ArmSeq,
}

impl ToTokens for ColExprCase {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(case) = &self.case {
            case.to_tokens(tokens);
        }
        self.arms.to_tokens(tokens)
    }
}

impl syn::parse::Parse for ColExprCase {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![match]>()?;
        let case = input.fork().parse::<ColExpr>()?;
        let case = if let ColExpr::Ident(_) = case {
            Some(Box::new(input.parse::<ColExpr>()?))
        } else {
            None
        };
        let arms = ArmSeq::parse_separated_nonempty(input)?;
        Ok(ColExprCase { case, arms })
    }
}

impl ColumnToSql for ColExprCase {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        let mut res = if let Some(ref case) = self.case {
            (*case).column_to_sql(ctx)?
        } else {
            "".into()
        };
        res.prepend_str("CASE ");
        for arm in &self.arms {
            res = res.add_no_comma(arm.column_to_sql(ctx)?);
        }
        res.append_str(" END");
        Ok(res)
    }
}

type ArmSeq = Punctuated<Arm, Token![,]>;

#[derive(Debug, Clone)]
struct Arm {
    lhs: ColExpr,
    rhs: ColExpr,
}

impl syn::parse::Parse for Arm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            ColExpr::Ident(IdentString::new(syn::Ident::new("_", input.span())))
        } else {
            ColExpr::parse(input)?
        };
        input.parse::<Token![=>]>()?;
        let rhs = ColExpr::parse(input)?;
        Ok(Self { lhs, rhs })
    }
}

impl ToTokens for Arm {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lhs.to_tokens(tokens);
        "=>".to_tokens(tokens);
        self.rhs.to_tokens(tokens);
    }
}

impl ColumnToSql for Arm {
    fn column_to_sql(
        &self,
        ctx: &mut crate::macros::Generator,
    ) -> Result<crate::macros::Fragment, crate::error::SqloError> {
        if let ColExpr::Ident(ref i) = self.lhs {
            if i.as_str() == "_" {
                let mut res = self.rhs.column_to_sql(ctx)?;
                res.prepend_str("ELSE ");
                return Ok(res);
            }
        }
        let mut res = self.lhs.column_to_sql(ctx)?;
        res.prepend_str("WHEN ");
        res.append_str(" THEN");
        res = res.add_no_comma(self.rhs.column_to_sql(ctx)?);
        Ok(res)
    }
}
