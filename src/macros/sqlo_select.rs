use proc_macro2::TokenStream;

use quote::quote;
use syn::{punctuated::Punctuated, Token};

use crate::{sqlo::Sqlo, sqlos::Sqlos, virtual_file::VirtualFile};

use super::{
    wwhere::{tokenizer::WhereTokenizer, where_generate_sql},
    SqlQuery,
};

mod kw {
    syn::custom_keyword!(order_by);
    syn::custom_keyword!(limit);
}

type PunctuatedExprComma = Punctuated<syn::Expr, Token![,]>;

pub struct SqloSelectParse {
    entity: syn::Ident,
    wwhere: Option<WhereTokenizer>,
    order_by: Option<PunctuatedExprComma>,
}

impl SqloSelectParse {
    fn new(ident: syn::Ident) -> Self {
        Self {
            entity: ident,
            wwhere: None,
            order_by: None,
        }
    }
}

// select![Maison where some_binary_ops order_by some,comma_separated,fields limit u32]
impl syn::parse::Parse for SqloSelectParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse sqlo struct
        let entity: syn::Ident = input
            .parse()
            .map_err(|_| syn::Error::new(input.span(), "Deriving Sqlo struct expected"))?;
        let mut res = SqloSelectParse::new(entity);

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

impl SqloSelectParse {
    fn expand(self, sqlos: &Sqlos) -> syn::Result<TokenStream> {
        let main = sqlos.get(&self.entity)?;
        if let Some(ref wwhere) = self.wwhere {
            let where_sql = where_generate_sql(&self.entity.to_string(), &sqlos, wwhere)?;
            return Ok(self.query(main, &where_sql));
        }
        Ok(quote!())
    }

    fn query(self, main: &Sqlo, wwhere: &SqlQuery) -> TokenStream {
        let columns = main.all_columns_as_query();
        let Sqlo {
            ident, tablename, ..
        } = main;
        let where_query = &wwhere.query;
        let where_params = &wwhere.params;
        let qquery = format!("SELECT {columns} FROM {tablename} {where_query}");
        quote! {
            sqlx::query_as!(#ident,#qquery, #(#where_params),*)
        }
    }
}

pub fn process_sqlo_select(input: SqloSelectParse) -> syn::Result<TokenStream> {
    let sqlos = VirtualFile::new().load()?;
    input.expand(&sqlos)
}

#[cfg(test)]
mod test_select_macro {
    use super::*;

    macro_rules! success_parse_sqlo_select_syntax {
        ($case:ident, $input:literal) => {
            paste::paste! {

                #[test]
                fn [<test_parse_select_syntax_ success $case>]() {
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
    fail_parse_sqlo_select_syntax!(not_where_after_entity, "Maison wrong", "`where` expected");
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
}
