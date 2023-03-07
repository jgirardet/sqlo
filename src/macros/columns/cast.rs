use std::fmt::Display;

use darling::util::IdentString;
use syn::{LitStr, Token};

use crate::{
    error::SqloError,
    macros::{Context, SqlQuery, SqlResult},
};

use super::{ColExpr, ColumnToSql};

#[derive(Debug)]
pub struct ColumnCast {
    pub expr: ColExpr,
    pub alias: AliasCast,
}

impl ColumnToSql for ColumnCast {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        ctx.context.push(Context::Cast);
        let expr = self.expr.column_to_sql(ctx)?;
        let res = expr.add_no_comma(self.alias.column_to_sql(ctx)?);
        ctx.context.pop();
        Ok(res)
    }
}

#[derive(Debug)]
pub enum AliasCast {
    Ident(IdentString),
    Literal(LitStr),
}

impl ColumnToSql for AliasCast {
    fn column_to_sql(&self, ctx: &mut SqlResult) -> Result<SqlQuery, SqloError> {
        match self {
            Self::Ident(ident) => {
                ctx.alias.insert(ident.clone(), ident.to_string());
                Ok(SqlQuery::from(format!(" as {ident}")))
            }
            Self::Literal(litstr) => {
                let re = regex_macro::regex!(r#"^(\w+)[?!]?(?::\w+(?:::\w+)*)?$"#);
                let alias_str = &litstr.value();
                if let Some(captures) = re.captures(alias_str) {
                    if let Some(alias) = captures.get(1) {
                        let ident: IdentString =
                            syn::Ident::new(alias.as_str(), litstr.span()).into();
                        let formated_alias_string = format!("\"{alias_str}\"");
                        ctx.alias.insert(ident, formated_alias_string.clone());
                        return Ok(format!(" as {formated_alias_string}").into());
                    }
                }
                Err(SqloError::new_spanned(litstr, "invalid alias format"))
            }
        }
    }
}

impl From<&syn::Ident> for AliasCast {
    fn from(ident: &syn::Ident) -> Self {
        AliasCast::Ident(IdentString::new(ident.clone()))
    }
}

impl Display for AliasCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AliasCast::Ident(i) => write!(f, "{}", i),
            AliasCast::Literal(l) => write!(f, "\"{}\"", l.value()),
        }
    }
}

impl syn::parse::Parse for AliasCast {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input.parse::<LitStr>() {
            Ok(l) => Ok(AliasCast::Literal(l)),
            Err(_) => match input.parse::<syn::Ident>() {
                Ok(i) => {
                    if input.peek(Token![?]) {
                        input.parse::<Token![?]>()?;
                        Ok(AliasCast::Literal(LitStr::new(&format!("{i}?"), i.span())))
                    } else if input.peek(Token![!]) {
                        input.parse::<Token![!]>()?;
                        Ok(AliasCast::Literal(LitStr::new(&format!("{i}!"), i.span())))
                    } else {
                        Ok(AliasCast::Ident(i.into()))
                    }
                }
                Err(_) => Err(input.error("as must be followed by identifier or string literal")),
            },
        }
    }
}
