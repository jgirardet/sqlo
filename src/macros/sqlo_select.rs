use darling::util::IdentString;
use proc_macro2::TokenStream;

use syn::{punctuated::Punctuated, Token};

use crate::virtual_file::VirtualFile;

use super::{wwhere::tokenizer::WhereTokenizer, Column, SqlResult};

mod kw {
    syn::custom_keyword!(order_by);
    syn::custom_keyword!(limit);
}

type PunctuatedExprComma = Punctuated<syn::Expr, Token![,]>;

pub struct SqloSelectParse {
    pub entity: IdentString,
    pub related: Option<IdentString>,
    pub customs: Vec<Column>,
    pub pk_value: Option<syn::Expr>,
    pub wwhere: Option<WhereTokenizer>,
    pub order_by: Option<PunctuatedExprComma>,
}

impl SqloSelectParse {
    fn new(ident: syn::Ident) -> Self {
        Self {
            entity: ident.into(),
            related: None,
            pk_value: None,
            wwhere: None,
            order_by: None,
            customs: Vec::default(),
        }
    }
}

// select![Maison where some_binary_ops order_by some,comma_separated,fields limit u32]
impl syn::parse::Parse for SqloSelectParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse first part: simple ident, ident[pk].related

        // Parse sqlo struct
        let entity: syn::Ident = input
            .parse()
            .map_err(|_| syn::Error::new(input.span(), "Deriving Sqlo struct expected"))?;
        let mut res = SqloSelectParse::new(entity);

        //related select
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            res.pk_value = Some(content.parse::<syn::Expr>()?);
            input.parse::<Token![.]>()?;
            res.related = Some(input.parse::<syn::Ident>()?.into());
        }

        // parse curtom column
        if !input.is_empty() && !input.peek(Token![where]) {
            let punct: Punctuated<Column, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
            res.customs = punct.into_iter().collect();
        }

        // parse where
        if !input.is_empty() {
            input
                .parse::<Token![where]>()
                .map_err(|_| syn::Error::new(input.span(), "`where` expected"))?;

            let wwhere = input.parse::<WhereTokenizer>()?;
            res.wwhere = Some(wwhere);
        }

        // parse order by
        if !input.is_empty() {
            input.parse::<kw::order_by>()?;
            let order_by: PunctuatedExprComma =
                syn::punctuated::Punctuated::parse_separated_nonempty(input).map_err(|_| {
                    syn::Error::new(input.span(), "comma seperated fields expected")
                })?;
            res.order_by = Some(order_by)
        }

        // parse limit
        // plus tard

        Ok(res)
    }
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
    success_parse_sqlo_select_syntax!(
        where_with_any_expr,
        r#"Maison where  1 == 1 && ([1,2,3].contains(3) || "fze".startswith('f'))"#
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
                    assert_eq!(syn::parse_str::<SqloSelectParse>($input).err().unwrap().to_string(),$err.to_string())
                }
            }
        };
    }

    fail_parse_sqlo_select_syntax!(empty, "", "Deriving Sqlo struct expected");
    fail_parse_sqlo_select_syntax!(
        not_irder_by_after_binaries,
        "Maison where 1 == 1 bla",
        "expected `order_by`"
    );
    fail_parse_sqlo_select_syntax!(
        not_comma_field_after_order_by,
        "Maison where 1 == 1 order_by",
        "comma seperated fields expected"
    );
    fail_parse_sqlo_select_syntax!(
        no_call_without_cast_allowed,
        "Maison count(id)",
        "column's expression should be followed by as"
    );
}
