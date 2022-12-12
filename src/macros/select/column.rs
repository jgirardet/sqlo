use darling::util::IdentString;
use syn::{
    BinOp, Expr, ExprBinary, ExprCast, ExprLit, ExprParen, ExprPath, Ident, Lit, LitStr, Member,
    Token,
};

use crate::query_builder::rust_op_to_sql_op;

use super::{
    kw,
    sql::{is_a_keyword, ToSql, SQL_FUNCTIONS},
    table::Tables,
};

#[derive(Debug)]
enum ColumnParse {
    Col(Ident),
    Field(Ident, Box<ColumnParse>),
    Call(Ident, Box<ColumnParse>),
    Operation(Box<ColumnParse>, BinOp, Box<ColumnParse>),
    Literal(Lit),
    Paren(Box<ColumnParse>),
}

impl syn::parse::Parse for ColumnParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let col = input.parse::<Expr>()?;
        ColumnParse::try_from(col)
    }
}
impl TryFrom<Expr> for ColumnParse {
    type Error = syn::Error;

    fn try_from(col: Expr) -> Result<Self, Self::Error> {
        match col {
            // simple field as column name
            Expr::Path(ref p) => {
                if let Some(ident) = p.path.get_ident() {
                    Ok(ColumnParse::Col(ident.clone()))
                } else {
                    return_error!(p, "Invalid identifier");
                }
            }
            Expr::Binary(ExprBinary {
                left, op, right, ..
            }) => {
                let lhs = ColumnParse::try_from(*left)?;
                let rhs = ColumnParse::try_from(*right)?;

                Ok(ColumnParse::Operation(Box::new(lhs), op, Box::new(rhs)))
            }
            Expr::Lit(ExprLit { lit, .. }) => Ok(ColumnParse::Literal(lit)),
            // sql function: sum, avg...
            Expr::Call(call) => {
                if let Expr::Path(ExprPath { ref path, .. }) = *call.func {
                    if let Some(p) = path.get_ident() {
                        let base = p.clone();
                        let content = match call.args.first() {
                            Some(expr) => ColumnParse::try_from(expr.clone())?,
                            _ => {
                                return_error!(call.args, "Argument not supported");
                            }
                        };
                        return Ok(ColumnParse::Call(base, Box::new(content)));
                    }
                }
                Err(syn::Error::new_spanned(call, "invalid call syntax"))
            }
            // using alias: a.myfield FROM mytable a
            Expr::Field(field) => {
                if let Expr::Path(ExprPath { path, .. }) = *field.base {
                    if let Some(ident) = path.get_ident() {
                        let base = ident.clone();

                        if let Member::Named(ident) = field.member {
                            Ok(ColumnParse::Field(base, Box::new(ColumnParse::Col(ident))))
                        } else {
                            return_error!(field.member, "not a valid field name");
                        }
                    } else {
                        return_error!(path, "struct/alias name is invalid");
                    }
                } else {
                    return_error!(field.base, "struct/alias name is invalid");
                }
            }
            // just handle 'as' error if lowercase
            Expr::Cast(ExprCast { as_token, .. }) => {
                return_error!(as_token, "Did you mean 'AS' ?")
            }
            // Parenthesis
            Expr::Paren(ExprParen { expr, .. }) => {
                Ok(ColumnParse::Paren(Box::new(ColumnParse::try_from(*expr)?)))
            }

            _ => return_error!(
                col,
                "Should be a field, strut.field, literal or sql function"
            ),
        }
    }
}

#[derive(Debug)]
pub struct Column {
    content: ColumnParse,
    alias: Option<Alias>,
}

impl syn::parse::Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content = input.parse()?;
        let mut alias = None;
        if input.peek(kw::AS) {
            alias = Some(input.parse::<Alias>()?);
        }
        Ok(Self { content, alias })
    }
}

impl Column {
    pub fn validate_columns(colums: Vec<Column>) -> syn::Result<Columns> {
        Ok(colums.into())
    }
}

impl ToSql for Column {
    fn to_sql(&self, tables: &Tables) -> syn::Result<String> {
        let alias_str = if let Some(alias) = &self.alias {
            match alias {
                Alias::Ident(ident) => format!(" AS {}", ident),
                Alias::String(string) => format!(" AS \"{}\"", string.value()),
            }
        } else {
            String::new()
        };
        Ok(format!("{}{alias_str}", &self.content.to_sql(tables)?))
    }
}

impl ColumnParse {
    pub fn to_sql(&self, tables: &Tables) -> syn::Result<String> {
        match *self {
            ColumnParse::Col(ref i) => {
                Ok(tables.field(&i.clone().into(), None)?.column.to_string())
            }
            ColumnParse::Literal(ref l) => match l {
                Lit::Str(s) => {
                    let res = format!("'{}'", s.value());
                    Ok(res)
                }
                Lit::Int(i) => Ok(i.base10_digits().to_string()),
                Lit::Float(f) => Ok(f.base10_digits().to_string()),
                Lit::Bool(b) => Ok(if b.value {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }),
                _ => return_error!(l, "Literal not supported"),
            },
            ColumnParse::Field(ref base, ref member) => {
                let table = tables.get(base)?;
                let member_ident: IdentString = if let ColumnParse::Col(ident) = member.as_ref() {
                    ident.clone().into()
                } else {
                    return_error!(base, "Invalid associated field");
                };
                if table.sqlo.field(&member_ident).is_some() {
                    let table_base = if table.alias.is_some() {
                        base.to_string()
                    } else {
                        table.sqlo.tablename.clone()
                    };
                    Ok(format!("{table_base}.{}", member.to_sql(&table.into())?))
                } else {
                    return_error!(
                        member_ident,
                        format!("Field not found in {}", table.ident())
                    )
                }
            }
            ColumnParse::Call(ref ident, ref content) => {
                let value = ident.to_string();
                if !SQL_FUNCTIONS.contains(&value.as_str()) {
                    if SQL_FUNCTIONS.contains(&value.to_uppercase().as_str()) {
                        return Err(syn::Error::new_spanned(
                            ident,
                            &format!("Did you mean '{}'", &value.to_uppercase()),
                        ));
                    }
                    return_error!(ident, "Sql function not supported")
                }
                Ok(format!("{ident}({})", content.to_sql(tables)?))
            }
            ColumnParse::Operation(ref lhs, op, ref rhs) => Ok(format!(
                "{} {} {}",
                lhs.to_sql(tables)?,
                rust_op_to_sql_op(&op),
                rhs.to_sql(tables)?
            )),
            ColumnParse::Paren(ref content) => Ok(format!("({})", content.to_sql(tables)?)),
        }
    }
}

#[derive(Debug, Default)]
pub struct Columns(Vec<Column>);

impl From<Vec<Column>> for Columns {
    fn from(columns: Vec<Column>) -> Self {
        Columns(columns)
    }
}

impl syn::parse::Parse for Columns {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut columns: Vec<Column> = vec![];
        while !is_a_keyword(input) {
            // a keyword
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            columns.push(input.parse::<Column>()?);
        }
        Column::validate_columns(columns)
    }
}

impl Columns {
    pub fn to_sql(&self, tables: &Tables) -> syn::Result<String> {
        let mut res = vec![];
        for c in &self.0 {
            res.push(c.to_sql(tables)?)
        }
        let rr = res.join(", ");
        Ok(rr)
    }
}

#[derive(Debug)]
pub enum Alias {
    Ident(Ident),
    String(LitStr),
}

impl syn::parse::Parse for Alias {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::AS>()?;
        if input.peek(Ident) {
            Ok(Alias::Ident(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Alias::String(input.parse::<LitStr>()?))
        } else {
            Err(input.error("Cast format not supported"))
        }
    }
}
