#![cfg(feature = "sqlite")]

use itertools::Itertools;
use syn::{parse_quote, Expr, Ident};

pub fn db_ident() -> Ident {
    parse_quote!(Sqlite)
}

pub fn db_sqlx_path() -> Expr {
    parse_quote!(sqlx::sqlite::Sqlite)
}

pub fn qmarks(nb: usize) -> String {
    (0..nb).map(|_| "?").join(",")
}

pub fn qmarks_with_col(cols: &[&str]) -> String {
    cols.iter().map(|c| format!("{c}=?")).join(",")
}

pub fn db_query_result_path() -> Expr {
    parse_quote!(sqlx::sqlite::SqliteQueryResult)
}

#[cfg(test)]
mod test_query_builder {
    use super::*;

    macro_rules! test_qmarks {
        ($nb:literal  $res:literal) => {
            paste::paste! {

                #[test]
                fn [<qmarks_sqlite_  $nb>]() {
                    assert_eq!(qmarks($nb),$res);
                }
            }
        };
    }

    test_qmarks!(1 "?" );
    test_qmarks!(2 "?,?");
    test_qmarks!(0 "" );

    macro_rules! test_qmarks_with_col {
        ($($col:literal),*; $res:literal) => {
            paste::paste!{

                #[test]
                fn [<qmarks_with_col_sqlite _ $($col)*>]() {
                    assert_eq!(
                        qmarks_with_col(&[$($col),*])
                        ,$res.to_string()
                    );
                }
            }
            };
    }

    test_qmarks_with_col!("bla","bli"; "bla=?,bli=?");
    test_qmarks_with_col!("bla"; "bla=?");
    test_qmarks_with_col!(; "");
}
