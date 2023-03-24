use darling::util::IdentString;

use super::{
    parse_bracketed, parse_dbg_symbol, parse_optional_field_member, parse_optional_where,
    parse_parenthezide, parse_sqlo_struct_ident, Assigns, Clauses, Column, Fetch, PkValue,
    QueryParser,
};

pub struct UpdateParser {
    #[cfg(debug_assertions)]
    debug: bool,
    entity: IdentString,
    pk_value: PkValue,
    related: Option<IdentString>,
    clauses: Clauses,
    assignments: Assigns,
    fetch: Fetch,
}

impl QueryParser for UpdateParser {
    fn debug(&self) -> bool {
        self.debug
    }

    fn entity(&self) -> &IdentString {
        &self.entity
    }

    fn related(&self) -> &Option<IdentString> {
        &self.related
    }

    fn assigns(&self) -> &Assigns {
        &self.assignments
    }

    fn custom_struct(&self) -> Option<IdentString> {
        None
    }

    fn pk_value(&self) -> PkValue {
        self.pk_value.clone()
    }

    fn clauses(&self) -> &Clauses {
        &self.clauses
    }

    fn columns(&self) -> &[Column] {
        panic!("Must not be used with Update")
    }

    fn fetch(&self) -> Fetch {
        self.fetch
    }
}

impl syn::parse::Parse for UpdateParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        #[cfg(debug_assertions)]
        let debug = input.call(parse_dbg_symbol)?;

        // parse fetch mode
        let fetch: Fetch = input.parse()?;

        // parse sqlo ident
        let entity = input.call(parse_sqlo_struct_ident)?;
        // or ident[pk] or  ident[pk].related or ident(instance) or ...
        let mut pk_value = input.call(parse_bracketed)?;
        if let PkValue::None = pk_value {
            pk_value = input.call(parse_parenthezide)?;
        }
        let related = input.call(parse_optional_field_member)?;

        // parse assignments
        let assignments = Assigns::parse(input)?;

        // where clause
        let mut clauses = Clauses::new();
        clauses.try_push(input, parse_optional_where)?;

        Ok(UpdateParser {
            debug,
            entity,
            related,
            assignments,
            pk_value,
            clauses,
            fetch,
        })
    }
}
