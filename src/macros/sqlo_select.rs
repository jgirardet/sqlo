use darling::util::IdentString;
use proc_macro2::TokenStream;

use syn::{parse::ParseStream, punctuated::Punctuated, Token};

use crate::virtual_file::VirtualFile;

use super::{Column, GroupBy, Limit, OrderBys, SqlResult, Where};

pub mod kw {
    syn::custom_keyword!(order_by);
    syn::custom_keyword!(limit);
    syn::custom_keyword!(page);
    syn::custom_keyword!(group_by);
}

pub struct SqloSelectParse {
    pub entity: IdentString,
    pub related: Option<IdentString>,
    pub customs: Vec<Column>,
    pub custom_struct: Option<IdentString>,
    pub pk_value: Option<syn::Expr>,
    pub wwhere: Option<Where>,
    pub order_by: Option<OrderBys>,
    pub limit: Option<Limit>,
    pub group_by: Option<GroupBy>,
}

impl SqloSelectParse {
    fn new(ident: syn::Ident) -> Self {
        Self {
            entity: ident.into(),
            related: None,
            pk_value: None,
            wwhere: None,
            order_by: None,
            limit: None,
            group_by: None,
            customs: Vec::default(),
            custom_struct: None,
        }
    }
}

// select![Maison where some_binary_ops order_by some,comma_separated,fields limit u32]
impl syn::parse::Parse for SqloSelectParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // First: parse cust struct
        let custom_struct = if input.peek2(Token![,]) {
            let custom_struct = input.parse::<syn::Ident>()?.into();
            input.parse::<Token![,]>()?;
            Some(custom_struct)
        } else {
            None
        };

        // then Parse first part: simple ident, ident[pk].related

        // Parse sqlo struct
        let entity: syn::Ident = input
            .parse()
            .map_err(|_| syn::Error::new(input.span(), "Deriving Sqlo struct expected"))?;
        let mut res = SqloSelectParse::new(entity);
        res.custom_struct = custom_struct; // reapply custom_struct

        //related select
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            res.pk_value = Some(content.parse::<syn::Expr>()?);
            input.parse::<Token![.]>()?;
            res.related = Some(input.parse::<syn::Ident>()?.into());
        }

        // parse curtom column
        if !input.is_empty() && next_is_not_a_keyword(&input) {
            let punct: Punctuated<Column, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
            res.customs = punct.into_iter().collect();
        }

        // parse where
        if !input.is_empty() && input.peek(Token![where]) {
            input
                .parse::<Token![where]>()
                .map_err(|_| syn::Error::new(input.span(), "`where` expected"))?;

            let wwhere = input.parse()?;
            res.wwhere = Some(wwhere);
        }

        // parse group by
        if !input.is_empty() && input.peek(kw::group_by) {
            res.group_by = Some(input.parse::<GroupBy>()?)
        }

        // parse order by
        if !input.is_empty() && input.peek(kw::order_by) {
            res.order_by = Some(input.parse::<OrderBys>()?)
        }

        // parse limit
        if !input.is_empty() && (input.peek(kw::limit) || input.peek(kw::page)) {
            res.limit = Some(input.parse::<Limit>()?)
        }

        Ok(res)
    }
}

pub fn next_is_not_a_keyword(input: &ParseStream) -> bool {
    !input.peek(Token![where])
        && !input.peek(kw::order_by)
        && !input.peek(kw::limit)
        && !input.peek(kw::page)
        && !input.peek(kw::group_by)
}

pub fn process_sqlo_select(input: SqloSelectParse) -> syn::Result<TokenStream> {
    let sqlos = VirtualFile::new().load()?;
    let sqlr = SqlResult::from_sqlo_parse(input, &sqlos)?;
    match sqlr.expand() {
        Ok(o) => Ok(o),
        Err(e) => Err(e.into()),
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
                    syn::parse_str::<SqloSelectParse>($input).unwrap();
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
    // success_parse_sqlo_select_syntax!(
    //     where_with_any_expr,
    //     r#"Maison where  1 == 1 && ([1,2,3].contains(3) || "fze".startswith('f'))"#
    // );
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
                    assert_eq!(syn::parse_str::<SqloSelectParse>($input).err().unwrap().to_string(),$err.to_string())
                }
            }
        };
    }

    fail_parse_sqlo_select_syntax!(empty, "", "Deriving Sqlo struct expected");
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
    fail_parse_sqlo_select_syntax!(
        no_call_without_cast_allowed,
        "Maison count(id)",
        "call expression must be followed by as"
    );
}
