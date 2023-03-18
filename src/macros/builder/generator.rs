use std::collections::HashMap;

use std::collections::HashSet;

use darling::util::IdentString;
use proc_macro2::TokenStream;

use crate::{error::SqloError, relations::Relation, sqlo::Sqlo, sqlos::Sqlos};

use super::mode::Mode;
use super::query_builder::QueryBuilder;
use super::QueryParser;
use super::{Context, Fragment, TableAliases};

pub struct Generator<'a> {
    pub main_sqlo: &'a Sqlo,
    pub sqlos: &'a Sqlos,
    pub aliases: HashMap<IdentString, String>,
    pub context: Vec<Context>,
    pub mode: Mode,
    pub related: Option<&'a Relation>,
    pub custom_struct: Option<IdentString>,
    pub tables: TableAliases<'a>,
    query_parts: QueryBuilder,
}

impl<'a> Generator<'a> {
    fn new(main_sqlo: &'a Sqlo, sqlos: &'a Sqlos, mode: Mode, tables: TableAliases<'a>) -> Self {
        Generator {
            sqlos,
            main_sqlo,
            mode,
            tables,
            aliases: HashMap::default(),
            related: Option::default(),
            query_parts: QueryBuilder::default(),
            custom_struct: None,
            context: Vec::default(),
        }
    }

    fn get_main_sqlo<T: QueryParser>(parsed: &T, sqlos: &'a Sqlos) -> Result<&'a Sqlo, SqloError> {
        if let Some(related) = parsed.related() {
            sqlos.get_by_relation(parsed.entity(), related)
        } else {
            sqlos.get(parsed.entity())
        }
    }
}

impl<'a> Generator<'a> {
    fn process_from(&mut self) {
        if !self.tables.contains(&self.main_sqlo.ident) {
            self.tables.insert_sqlo(&self.main_sqlo.ident)
        }
    }

    fn set_relation_if_related<T: QueryParser>(&mut self, parsed: &T) -> Result<(), SqloError> {
        if let Some(related) = parsed.related() {
            self.related = Some(self.sqlos.get_relation(parsed.entity(), related)?);
        }
        Ok(())
    }

    fn parse<T: QueryParser>(&mut self, parsed: T) -> Result<(), SqloError> {
        // preleminary
        self.process_from();
        self.set_relation_if_related(&parsed)?;
        // query_parts
        let mut qp = QueryBuilder::default();
        qp.parse(&parsed, self)?;
        self.query_parts = qp;
        // custom_struct
        self.custom_struct = parsed.custom_struct().clone();
        Ok(())
    }
}

// publique interface
impl<'a> Generator<'_> {
    pub fn from_sqlo_query_parse<T>(
        mode: Mode,
        parsed: T,
        sqlos: &'a Sqlos,
        subquery: bool,
        table_aliases: TableAliases<'a>,
    ) -> Result<Generator<'a>, SqloError>
    where
        T: QueryParser,
    {
        let main_sqlo = Generator::get_main_sqlo(&parsed, sqlos)?;
        let mut generator = Generator::new(main_sqlo, sqlos, mode, table_aliases);
        if subquery {
            generator.context.push(Context::SubQuery);
        }
        generator.parse(parsed)?;
        Ok(generator)
    }

    pub fn expand(&self) -> Result<TokenStream, SqloError> {
        let query = self.query_parts.query(self)?;
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            println!("query: {}", &query);
        } else if std::env::var("SQLO_DEBUG_QUERY_ALL").is_ok() {
            let dd = format!(
                "query: {} \n args: {:?}",
                &query, &self.query_parts.arguments
            );
            println!("{}", dd);
        }
        let ident = if let Some(ident) = &self.custom_struct {
            ident
        } else {
            &self.main_sqlo.ident
        };
        let arguments = self.arguments();

        if self.query_parts.customs && self.custom_struct.is_none() {
            Ok(quote::quote! {
                sqlx::query!(#query, #(#arguments),*)
            })
        } else {
            Ok(quote::quote! {
                sqlx::query_as!(#ident,#query, #(#arguments),*)
            })
        }
    }
    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        println!(
            "query: {} \nargs: {:?}",
            self.query().unwrap_or_else(|e| e.to_string()),
            self.arguments()
        );
    }

    pub fn arguments(&self) -> &[syn::Expr] {
        self.query_parts.arguments.as_slice()
    }

    pub fn query(&self) -> Result<String, SqloError> {
        self.query_parts.query(self)
    }
}

impl<'a> TryFrom<Generator<'a>> for Fragment {
    type Error = SqloError;

    fn try_from(result: Generator<'a>) -> Result<Self, Self::Error> {
        Ok(Fragment {
            query: result.query()?,
            params: result.arguments().into(),
            joins: HashSet::default(),
        })
    }
}
