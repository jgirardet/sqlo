use proc_macro2::TokenStream;
use syn::{Ident, Token};

use super::{column::Columns, distinct::Distinct, sql::ToSql, table::Tables};

#[derive(Debug, Default)]
pub struct Select {
    query_as: Option<Ident>,
    distinct: Distinct,
    columns: Columns,
    tables: Tables,
}

impl Select {
    fn validate(self) -> syn::Result<Self> {
        Ok(self)
    }
}

impl syn::parse::Parse for Select {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut res = Select::default();

        if input.peek2(Token![;]) {
            res.query_as = input.parse()?;
            input.parse::<Token![;]>()?;
        }

        //distinct
        res.distinct = input.parse()?;
        //parse columns,  count, max, ....
        res.columns = input.parse()?;
        // parse from table
        res.tables = input.parse()?;

        res.validate()
    }
}

impl Select {
    fn to_sql(&self) -> syn::Result<String> {
        let Select {
            distinct,
            columns,
            tables,
            ..
        } = self;
        let distinct = distinct.to_sql(&self.tables)?;
        let columns = columns.to_sql(&self.tables)?;
        let tables = tables.to_sql(&self.tables)?;
        Ok(format!("SELECT {distinct} {columns} FROM {tables}").replace("  ", " "))
    }

    pub fn expand(self) -> syn::Result<TokenStream> {
        let target_struct = if let Some(target) = &self.query_as {
            target
        } else {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Not implemented without query as",
            ));
        };
        let sql = self.to_sql()?;

        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&sql);
        }
        let res = quote::quote! {
            sqlx::query_as!(#target_struct, #sql)
        };
        Ok(res)
    }
}

#[cfg(test)]
mod test_parse_select_macro {
    use super::*;

    macro_rules! test_select_syntax {
        ($case:ident, $input:literal, $success:literal) => {
            paste::paste! {

                #[test]
                fn [<test_select_syntax_ $case>]() {
                    if $success {
                        syn::parse_str::<Select>($input).unwrap();
                    } else {
                        let res = syn::parse_str::<Select>($input);
                        assert!(res.is_err())
                    }
                }
            }
        };
    }

    test_select_syntax!(query_as, "MyStruct ; id FROM Aaa", true);
    test_select_syntax!(distinct, "DISTINCT id FROM Aaa", true);
    test_select_syntax!(ident, "id FROM Aaa", true);
    test_select_syntax!(func, "count(id) FROM Aaa", true);
    test_select_syntax!(field, "a.b FROM Aaa", true);
    test_select_syntax!(comma_seprated_column, "a.b, id, count(id) FROM Aaa", true);
    test_select_syntax!(table_alias, "id FROM Aaa a", true);
    test_select_syntax!(two_table_2_alias, "id FROM Aaa a,Bbb b", true);
    test_select_syntax!(table_alias_used_in_columns, "a.id FROM Aaa a", true);
    test_select_syntax!(
        table_alias_used_in_columns_with_after,
        "a.id FROM Aaa JOIN",
        false
    );

    macro_rules! select_to_sql {
        ($case:ident, $input:literal, $result:literal) => {
            paste::paste! {

            #[test]
            fn [<select_to_sql_ $case>]() {
                  let res = syn::parse_str::<Select>($input).unwrap().to_sql().unwrap();
                    assert_eq!(res, $result.to_string())
                }
            }
        };
    }

    select_to_sql!(
        distinct,
        "DISTINCT id FROM Aaa",
        "SELECT DISTINCT id FROM aaa"
    );
    select_to_sql!(ident, "id FROM Aaa", "SELECT id FROM aaa");
    select_to_sql!(func, "COUNT(id) FROM Aaa", "SELECT COUNT(id) FROM aaa");
    select_to_sql!(field, "Aaa.id FROM Aaa", "SELECT aaa.id FROM aaa");
    select_to_sql!(alias, "id FROM Aaa a", "SELECT id FROM aaa a");
    select_to_sql!(alias_as_field, "a.id FROM Aaa a", "SELECT a.id FROM aaa a");
    select_to_sql!(
        alias_as_mixed_field,
        "a.id, a.fstring, Bbb.fi32, Bbb.uu FROM Aaa a, Bbb",
        "SELECT a.id, a.fstring, bbb.fi32, bbb.uu FROM aaa a, bbb"
    );
    select_to_sql!(
        alias_as_mixed_field_two_alias,
        "a.id, a.fstring, b.fi32, b.uu FROM Aaa a, Bbb b",
        "SELECT a.id, a.fstring, b.fi32, b.uu FROM aaa a, bbb b"
    );
    select_to_sql!(change_col, "fi32 FROM Aaa", "SELECT fi32col FROM aaa");
    select_to_sql!(
        alias_as_mixed_field_chang_column_name,
        "a.id, a.fi32, Bbb.fstring, Bbb.uu FROM Aaa a, Bbb",
        "SELECT a.id, a.fi32col, bbb.fstringcol, bbb.uu FROM aaa a, bbb"
    );
    select_to_sql!(string_literal, r#""bla" FROM Aaa"#, "SELECT 'bla' FROM aaa");
    select_to_sql!(
        string_type_inforce,
        r#""bla" AS "b:_" FROM Aaa"#,
        r#"SELECT 'bla' AS "b:_" FROM aaa"#
    );
    select_to_sql!(
        simple_operation,
        "id + 3 FROM Aaa",
        "SELECT id + 3 FROM aaa"
    );
    select_to_sql!(parenthes, "(1 + id) FROM Aaa", "SELECT (1 + id) FROM aaa");
    select_to_sql!(
        logic_operator,
        "1==1 && 1!=0 FROM Aaa",
        "SELECT 1 = 1 AND 1 <> 0 FROM aaa"
    );
}
