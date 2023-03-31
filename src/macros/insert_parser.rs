use darling::util::IdentString;

use super::{
    parse_dbg_symbol, parse_sqlo_struct_ident, Assigns, Clauses, Column, Fetch, PkValue,
    QueryParser,
};

pub struct InsertParser {
    #[cfg(debug_assertions)]
    debug: bool,
    entity: IdentString,
    assignments: Assigns,
    fetch: Fetch,
}

impl QueryParser for InsertParser {
    fn debug(&self) -> bool {
        self.debug
    }

    fn entity(&self) -> &IdentString {
        &self.entity
    }

    fn related(&self) -> &Option<IdentString> {
        &None
    }

    fn assigns(&self) -> &Assigns {
        &self.assignments
    }

    fn custom_struct(&self) -> Option<IdentString> {
        None
    }

    fn pk_value(&self) -> PkValue {
        PkValue::None
    }

    fn clauses(&self) -> &Clauses {
        unreachable!("No clause with insert statment")
    }

    fn columns(&self) -> &[Column] {
        unreachable!("Must not be used with insert statment")
    }

    fn fetch(&self) -> Fetch {
        self.fetch
    }
}

impl syn::parse::Parse for InsertParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        #[cfg(debug_assertions)]
        let debug = input.call(parse_dbg_symbol)?;

        // parse fetch mode
        let fetch: Fetch = input.parse()?;

        // parse sqlo ident
        let entity = input.call(parse_sqlo_struct_ident)?;

        let assignments = Assigns::parse(input)?;

        Ok(InsertParser {
            debug,
            entity,
            assignments,
            fetch,
        })
    }
}
