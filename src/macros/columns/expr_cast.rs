use std::{fmt::Display, str::FromStr};

use darling::util::IdentString;
use syn::{LitStr, Token};

use super::ColExpr;
use crate::{
    macros::{ColumnToSql, Context, Fragment, Generator},
    SqloError,
};

#[derive(Debug, Clone)]
pub struct ColumnCast {
    pub expr: ColExpr,
    pub alias: AliasCast,
}

impl ColumnToSql for ColumnCast {
    fn column_to_sql(&self, ctx: &mut Generator) -> Result<Fragment, SqloError> {
        ctx.context.push(Context::Cast);
        let expr = self.expr.column_to_sql(ctx)?;

        // expr needed in column_to_sql for aliascast so we put it here
        let alias: Fragment = match &self.alias {
            AliasCast::Ident(ident) => {
                ctx.aliases.insert(ident.clone(), ident.to_string());
                Fragment::from(format!(" as {ident}"))
            }
            AliasCast::Literal(litstr) => {
                let re = regex_macro::regex!(r#"^(\w+)[?!]?(?::\w+(?:::\w+)*)?$"#);
                let alias_str = &litstr.value();
                if let Some(captures) = re.captures(alias_str) {
                    if let Some(alias) = captures.get(1) {
                        let ident: IdentString =
                            syn::Ident::new(alias.as_str(), litstr.span()).into();
                        let formated_alias_string = format!("\"{alias_str}\"");
                        ctx.aliases.insert(ident, expr.query.clone());
                        Fragment::from(format!(" as {formated_alias_string}"))
                    } else {
                        return Err(SqloError::new_spanned(litstr, "invalid alias format"));
                    }
                } else {
                    return Err(SqloError::new_spanned(litstr, "invalid alias format"));
                }
            }
        };
        let res = expr.add_no_comma(alias);
        ctx.context.pop();
        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub enum AliasCast {
    Ident(IdentString),
    Literal(LitStr),
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

impl FromStr for AliasCast {
    type Err = SqloError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AliasCast::Literal(LitStr::new(
            s,
            proc_macro2::Span::call_site(),
        )))
    }
}
