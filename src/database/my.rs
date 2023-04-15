#![cfg(feature = "mysql")]

use itertools::Itertools;
use syn::{parse_quote, Expr, Ident};

pub fn db_ident() -> Ident {
    parse_quote!(MySql)
}

pub fn db_sqlx_path() -> Expr {
    parse_quote!(sqlx::mysql::MySql)
}

pub fn qmarks(nb: usize) -> String {
    (0..nb).map(|_| "?").join(",")
}

pub fn qmarks_with_col(_: usize, cols: &[&str]) -> String {
    cols.iter().map(|c| format!("{c}=?")).join(",")
}

pub fn db_query_result_path() -> Expr {
    parse_quote!(sqlx::mysql::MySqlQueryResult)
}

#[cfg(test)]
mod test_database_sqlite {

    test_qmarks!(1 "?" );
    test_qmarks!(2 "?,?");
    test_qmarks!(0 "" );

    test_qmarks_with_col!("bla","bli"; "bla=?,bli=?");
    test_qmarks_with_col!("bla"; "bla=?");
    test_qmarks_with_col!(; "");
}
