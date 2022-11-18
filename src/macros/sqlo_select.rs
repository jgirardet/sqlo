use proc_macro2::TokenStream;
use std::fmt::Write;

use quote::quote;
use syn::{punctuated::Punctuated, Token};

use crate::{relations::Relation, sqlo::Sqlo, sqlos::Sqlos, virtual_file::VirtualFile};

use super::{sql_query::SqlQuery, wwhere::tokenizer::WhereTokenizer};

mod kw {
    syn::custom_keyword!(order_by);
    syn::custom_keyword!(limit);
}

type PunctuatedExprComma = Punctuated<syn::Expr, Token![,]>;

pub struct SqloSelectParse {
    entity: syn::Ident,
    related: Option<syn::Ident>,
    instance: Option<syn::Expr>,
    wwhere: Option<WhereTokenizer>,
    order_by: Option<PunctuatedExprComma>,
}

impl SqloSelectParse {
    fn new(ident: syn::Ident) -> Self {
        Self {
            entity: ident,
            related: None,
            instance: None,
            wwhere: None,
            order_by: None,
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
            res.instance = Some(content.parse::<syn::Expr>()?);
            input.parse::<Token![.]>()?;
            res.related = Some(input.parse::<syn::Ident>()?);
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

impl SqloSelectParse {
    fn expand(&self, sqlos: &Sqlos) -> syn::Result<TokenStream> {
        let main = sqlos.get(&self.entity)?;

        if let Some(ref related) = self.related {
            self.expand_related(related, main, sqlos)
        } else {
            self.expand_simple(main, sqlos)
        }
    }

    fn expand_simple(&self, main_sqlo: &Sqlo, sqlos: &Sqlos) -> syn::Result<TokenStream> {
        let wwhere_sql =
            SqlQuery::try_from_option_where_tokenizer(self.wwhere.clone(), sqlos, main_sqlo)?;
        Ok(SqloSelectParse::query(main_sqlo, wwhere_sql))
    }

    fn expand_related(
        &self,
        related: &syn::Ident,
        main_sqlo: &Sqlo,
        sqlos: &Sqlos,
    ) -> syn::Result<TokenStream> {
        let Relation::ForeignKey(relation) = sqlos.relations.find(&main_sqlo.ident, related)?;
        let related_sqlo = sqlos.get(&relation.from)?;
        let mut wwhere_sql =
            SqlQuery::try_from_option_where_tokenizer(self.wwhere.clone(), sqlos, related_sqlo)?;
        let prefix = if wwhere_sql.query.is_empty() {
            "WHERE "
        } else {
            " AND "
        };
        write!(
            wwhere_sql.query,
            "{}{}=?",
            prefix,
            &relation.get_from_column(sqlos)
        )
        .expect("Error formatting where related where query");

        wwhere_sql.params.push(self.instance.clone().unwrap()); // ok since related exists only if instance is parsed.
        Ok(SqloSelectParse::query(related_sqlo, wwhere_sql))
    }

    fn query(from: &Sqlo, wwhere: SqlQuery) -> TokenStream {
        let Sqlo {
            ident, tablename, ..
        } = from;
        let columns = from.all_columns_as_query();
        let (where_query, where_params) = (wwhere.query, wwhere.params);

        let qquery = format!("SELECT DISTINCT {columns} FROM {tablename} {where_query}");
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&qquery);
        }

        // build res tokenstream
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
