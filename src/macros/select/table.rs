use darling::util::IdentString;
use syn::{Ident, Token};

use crate::{field::Field, sqlo::Sqlo, virtual_file::VirtualFile};

use super::{kw, sql::ToSql};

#[derive(Debug, Clone)]
pub struct Table {
    pub sqlo: Sqlo,
    pub alias: Option<Ident>,
}

impl Table {
    pub fn ident(&self) -> &IdentString {
        &self.sqlo.ident
    }
}

impl ToSql for Table {
    fn to_sql(&self, _: &Tables) -> syn::Result<String> {
        let alias = match &self.alias {
            Some(x) => format!(" {x}"),
            None => "".to_string(),
        };
        Ok(format!("{}{alias}", self.sqlo.tablename))
    }
}

#[derive(Debug)]
struct TableParse {
    ident: Ident,
    alias: Option<Ident>,
}

impl syn::parse::Parse for TableParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let alias = if !input.is_empty() && TableParse::continue_parse_table(&input) {
            input.parse::<Option<Ident>>()?
        } else {
            None
        };
        Ok(TableParse { ident, alias })
    }
}

impl TableParse {
    pub fn continue_parse_table(input: &syn::parse::ParseStream) -> bool {
        if input.peek(kw::JOIN) || input.peek(kw::WHERE) {
            return false;
        }
        true
    }

    pub fn validate_tables(tables: Vec<TableParse>) -> syn::Result<Tables> {
        let mut res = Vec::with_capacity(tables.len());
        let sqlos = VirtualFile::new().load()?;
        for tab in tables {
            if let Ok(sqlo) = sqlos.get(tab.ident.to_string()) {
                let mut sqlo = sqlo.clone();
                let mut ident = sqlo.ident.as_ident().clone();
                ident.set_span(tab.ident.span());
                sqlo.ident = ident.into();
                res.push(Table {
                    sqlo,
                    alias: tab.alias,
                });
            } else {
                return Err(syn::Error::new_spanned(
                    tab.ident,
                    "Should be an Sqlo's derived struct",
                ));
            }
        }

        Ok(res.into())
    }
}

#[derive(Debug, Default)]
pub struct Tables(Vec<Table>);

impl Tables {
    pub fn field(&self, field_name: &IdentString, tab: Option<&Ident>) -> syn::Result<&Field> {
        for t in &self.0 {
            if let Some(field) = t.sqlo.field(field_name) {
                if let Some(ident) = tab {
                    if t.ident().as_ident() == ident {
                        return Ok(field);
                    }
                } else {
                    return Ok(field);
                }
            }
        }
        let tables_ident = if let Some(ident) = tab {
            ident.to_string()
        } else {
            self.0
                .iter()
                .map(|f| f.ident().to_string())
                .collect::<Vec<_>>()
                .join(",")
        };
        Err(syn::Error::new_spanned(
            field_name,
            format!("Field not found in {tables_ident}"),
        ))
    }

    pub fn get(&self, name: &Ident) -> syn::Result<&Table> {
        for t in &self.0 {
            if name == &t.sqlo.ident {
                return Ok(t);
            } else if let Some(alias) = &t.alias {
                if name == alias {
                    return Ok(t);
                }
            }
        }
        Err(syn::Error::new_spanned(
            name,
            "No Sqlo's Struct or alias found with identifier",
        ))
    }
}

impl From<Vec<Table>> for Tables {
    fn from(tables: Vec<Table>) -> Self {
        Tables(tables)
    }
}

impl From<&Table> for Tables {
    fn from(t: &Table) -> Self {
        Tables(vec![t.clone()])
    }
}

impl syn::parse::Parse for Tables {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut table_parses = vec![];
        if !(input.peek(kw::FROM)) {
            if input.peek(kw::from) {
                return Err(syn::Error::new(input.span(), "Did you mean 'FROM' ?"));
            } else {
                return Err(syn::Error::new(input.span(), "FROM expected"));
            }
        }
        input.parse::<kw::FROM>()?;
        while !input.is_empty() && TableParse::continue_parse_table(&input) {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            table_parses.push(input.parse::<TableParse>()?);
        }
        TableParse::validate_tables(table_parses)
    }
}

impl ToSql for Tables {
    fn to_sql(&self, tables: &Tables) -> syn::Result<String> {
        let mut res = vec![];
        for t in &self.0 {
            res.push(t.to_sql(tables)?)
        }
        Ok(res.join(", "))
    }
}
