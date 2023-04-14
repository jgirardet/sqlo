use std::collections::HashMap;

use darling::util::IdentString;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{error::SqloError, relations::Relation, sqlo::Sqlo, sqlos::Sqlos};

use super::expand_insert;
use super::expand_select;
use super::expand_update;
use super::mode::Mode;
use super::query_builder::QueryBuilder;
use super::Arguments;
use super::Fetch;
use super::PkValue;
use super::QueryParser;
use super::WhichMacro;
use super::{Context, TableAliases};

pub struct Generator<'a> {
    pub main_sqlo: &'a Sqlo,
    pub sqlos: &'a Sqlos,
    pub aliases: HashMap<IdentString, String>,
    pub context: Vec<Context>,
    pub mode: Mode,
    pub related: Option<&'a Relation>,
    pub custom_struct: Option<IdentString>,
    pub tables: TableAliases<'a>,
    pub fetch: Fetch,
    pk_value: PkValue,
    pub query_parts: QueryBuilder,
    pub arguments: Arguments,
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
            fetch: Fetch::default(),
            pk_value: PkValue::None,
            arguments: Arguments::default(),
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

    fn process_fetch<T: QueryParser>(&mut self, parsed: &T) {
        self.fetch = parsed.fetch()
    }

    fn process_pk_value<T: QueryParser>(&mut self, parsed: &T) {
        self.pk_value = parsed.pk_value();
    }

    fn parse<T: QueryParser>(&mut self, parsed: T) -> Result<(), SqloError> {
        // preleminary
        self.process_from();
        self.set_relation_if_related(&parsed)?;
        self.process_fetch(&parsed);
        self.custom_struct = parsed.custom_struct();
        self.process_pk_value(&parsed);
        // query_parts
        let mut qp = QueryBuilder::default();
        qp.parse(&parsed, self)?;
        self.query_parts = qp;
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

    pub fn expand(&self, #[cfg(debug_assertions)] debug: bool) -> Result<TokenStream, SqloError> {
        let initial_query = self.query_parts.query(self)?;
        let arguments = self.arguments.as_result(&initial_query);
        let query = self.format_query(&initial_query);
        let fetch = self.fetch;
        let ident = if let Some(ident) = &self.custom_struct {
            ident
        } else {
            &self.main_sqlo.ident
        };

        #[cfg(debug_assertions)]
        self.debug(&query, debug);

        match self.mode {
            Mode::Select => Ok(expand_select(
                fetch,
                ident,
                query,
                arguments.as_slice(),
                WhichMacro::for_select(self),
            )),
            Mode::Update => {
                let move_instance = if let PkValue::Parenthezide(instance) = &self.pk_value {
                    quote! {let #instance = #instance;}
                } else {
                    TokenStream::new()
                };
                Ok(expand_update(
                    fetch,
                    ident,
                    query,
                    arguments.as_slice(),
                    move_instance,
                ))
            }
            Mode::Insert => Ok(expand_insert(
                fetch,
                ident,
                query,
                arguments.as_slice(),
                self.main_sqlo,
            )),
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self, query: &str, debug: bool) {
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            println!("query: {}", &query);
        } else if std::env::var("SQLO_DEBUG_QUERY_ALL").is_ok() {
            let dd = format!("query: {} \n args: {:?}", &query, &self.arguments);
            println!("{}", dd);
        } else if debug {
            println!("query: {} \nargs: {:?}", query, self.arguments);
        }
    }

    #[cfg(feature = "postgres")]
    fn format_query(&self, query: &str) -> String {
        query.to_string()
    }

    #[cfg(not(feature = "postgres"))]
    fn format_query(&self, query: &str) -> String {
        let res = query.to_string();
        regex_macro::regex!(r"\$\d+")
            .replace_all(&res, "?")
            .to_string()
    }

    pub fn raw_query(&self) -> Result<String, SqloError> {
        self.query_parts.query(self)
    }
}
