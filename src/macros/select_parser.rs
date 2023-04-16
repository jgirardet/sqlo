use darling::util::IdentString;

#[cfg(debug_assertions)]
use super::parse_dbg_symbol;
use super::{parse_bracketed, parse_optional_field_member, Fetch, PkValue, QueryParser};

use crate::macros::{
    parse_optional_columns, parse_optional_group_by, parse_optional_having,
    parse_optional_ident_with_comma, parse_optional_limit_page, parse_optional_order_by,
    parse_optional_where, parse_sqlo_struct_ident,
};

use crate::macros::{Clauses, Column};

#[derive(Debug)]
pub struct SelectParser {
    #[cfg(debug_assertions)]
    debug: bool,
    entity: IdentString,
    related: Option<IdentString>,
    customs: Vec<Column>,
    custom_struct: Option<IdentString>,
    pk_value: PkValue,
    clauses: Clauses,
    fetch: Fetch,
}

impl syn::parse::Parse for SelectParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        #[cfg(debug_assertions)]
        let debug = input.call(parse_dbg_symbol)?;

        // parse fetch type
        let fetch = input.parse()?;

        // First: parse cust struct
        let custom_struct = input.call(parse_optional_ident_with_comma)?;

        // then parse sqlo ident
        let entity = input.call(parse_sqlo_struct_ident)?;
        // or  ident[pk].related
        let pk_value = input.call(parse_bracketed)?;
        let related = input.call(parse_optional_field_member)?;

        // parse optional custom colums
        let customs = input
            .call(parse_optional_columns)?
            .map(|x| x.into_iter().collect())
            .unwrap_or_default();

        // rest of the clauses
        let mut clauses = Clauses::new();
        clauses.try_push(input, parse_optional_where)?;
        clauses.try_push(input, parse_optional_group_by)?;
        clauses.try_push(input, parse_optional_having)?;
        clauses.try_push(input, parse_optional_order_by)?;
        clauses.try_push(input, parse_optional_limit_page)?;

        Ok(SelectParser {
            debug,
            entity,
            related,
            customs,
            custom_struct,
            pk_value,
            clauses,
            fetch,
        })
    }
}

impl QueryParser for SelectParser {
    fn debug(&self) -> bool {
        self.debug
    }

    fn entity(&self) -> &IdentString {
        &self.entity
    }

    fn related(&self) -> &Option<IdentString> {
        &self.related
    }

    fn columns(&self) -> &[Column] {
        &self.customs
    }

    fn custom_struct(&self) -> Option<IdentString> {
        self.custom_struct.clone()
    }

    fn pk_value(&self) -> PkValue {
        self.pk_value.clone()
    }

    fn clauses(&self) -> &Clauses {
        &self.clauses
    }

    fn assigns(&self) -> &super::Assigns {
        panic!("Assign must not be used with Select")
    }

    fn fetch(&self) -> Fetch {
        self.fetch
    }
}

#[cfg(test)]
mod test_sqlo_select_macro {
    use super::*;

    macro_rules! success_parse_sqlo_select_syntax {
        ($case:ident, $input:literal) => {
            paste::paste! {

                #[test]
                fn [<test_parse_select_syntax_ success_ $case>]() {
                    syn::parse_str::<SelectParser>($input).unwrap();
                }
            }
        };
    }

    success_parse_sqlo_select_syntax!(all_via_struct_ident, "Maison");
    success_parse_sqlo_select_syntax!(unique_where, "Maison where 1 == 1");
    success_parse_sqlo_select_syntax!(some_where, "Maison where 1 == 1 || 2 == 2");
    success_parse_sqlo_select_syntax!(
        where_with_parenthese,
        "Maison where  1 == 1 && (33 ==3 || 2 == 2)"
    );
    success_parse_sqlo_select_syntax!(order_by, r#"Maison where  1 == 1 && 2 == 2 order_by bla"#);
    success_parse_sqlo_select_syntax!(
        order_by_many,
        r#"Maison where  1 == 1 && 2 == 2 order_by bla,bli"#
    );
    success_parse_sqlo_select_syntax!(ident_related, "Maison[1].related");

    success_parse_sqlo_select_syntax!(unique_call, "Maison count(id) as bla");
    success_parse_sqlo_select_syntax!(unique_col_identifier, "Maison id");
    success_parse_sqlo_select_syntax!(call_plus_col, "Maison id, count(id) as bla");

    macro_rules! fail_parse_sqlo_select_syntax {
        ($case:ident, $input:literal, $err:literal) => {
            paste::paste! {

                #[test]
                fn [<test_parse_select_syntax_ fail $case>]() {
                    assert_eq!(syn::parse_str::<SelectParser>($input).err().unwrap().to_string(),$err.to_string())
                }
            }
        };
    }

    fail_parse_sqlo_select_syntax!(
        empty,
        "dbg!",
        "unexpected end of input, Derived Sqlo struct expected"
    );
    fail_parse_sqlo_select_syntax!(
        not_irder_by_after_binaries,
        "Maison where 1 == 1 bla",
        "unexpected token"
    );
    fail_parse_sqlo_select_syntax!(
        not_comma_field_after_order_by,
        "Maison where 1 == 1 order_by",
        "unexpected end of input, Sqlo: Invalid input"
    );
}
