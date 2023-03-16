use darling::util::IdentString;

#[cfg(debug_assertions)]
use super::parse_dbg_symbol;

use super::{
    parse_field_member, parse_optional_bracketed, parse_optional_columns, parse_optional_group_by,
    parse_optional_having, parse_optional_ident_with_comma, parse_optional_limit_page,
    parse_optional_order_by, parse_optional_where, parse_sqlo_struct_ident,
    query_parser::QueryParser,
};

use crate::macros::{Clauses, Column};

#[derive(Debug)]
pub struct SelectQueryParse {
    #[cfg(debug_assertions)]
    debug: bool,
    entity: IdentString,
    related: Option<IdentString>,
    customs: Vec<Column>,
    custom_struct: Option<IdentString>,
    pk_value: Option<syn::Expr>,
    clauses: Clauses,
}

impl syn::parse::Parse for SelectQueryParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        #[cfg(debug_assertions)]
        let debug = input.call(parse_dbg_symbol)?;

        // First: parse cust struct
        let custom_struct = input.call(parse_optional_ident_with_comma)?;

        // then parse sqlo ident
        let entity = input.call(parse_sqlo_struct_ident)?;
        // or  ident[pk].related
        let pk_value = input.call(parse_optional_bracketed)?;
        let related = match pk_value.is_some() {
            true => Some(input.call(parse_field_member)?),
            false => None,
        };

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

        Ok(SelectQueryParse {
            debug,
            entity,
            related,
            customs,
            custom_struct,
            pk_value,
            clauses,
        })
    }
}

impl QueryParser for SelectQueryParse {
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

    fn custom_struct(&self) -> &Option<IdentString> {
        &self.custom_struct
    }

    fn pk_value(&self) -> &Option<syn::Expr> {
        &self.pk_value
    }

    fn clauses(&self) -> &Clauses {
        &self.clauses
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
                    syn::parse_str::<SelectQueryParse>($input).unwrap();
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
                    assert_eq!(syn::parse_str::<SelectQueryParse>($input).err().unwrap().to_string(),$err.to_string())
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
