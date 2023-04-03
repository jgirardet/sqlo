#![cfg(feature = "postgres")]

use itertools::Itertools;
use syn::{parse_quote, Expr, Ident};

pub fn db_ident() -> Ident {
    parse_quote!(Postgres)
}

pub fn db_sqlx_path() -> Expr {
    parse_quote!(sqlx::postgres::Postgres)
}

pub fn qmarks(nb: usize) -> String {
    (1..=nb).map(|x| format!("${x}")).join(",")
}

pub fn qmarks_with_col(skip: usize, cols: &[&str]) -> String {
    cols.iter()
        .enumerate()
        .map(|(x, c)| {
            let z = x + 1 + skip;
            format!("{c}=${z}")
        })
        .join(",")
}

pub fn db_query_result_path() -> Expr {
    parse_quote!(sqlx::postgres::PgQueryResult)
}

#[cfg(test)]
mod test_database_sqlite {

    test_qmarks!(1 "$1" );
    test_qmarks!(2 "$1,$2");
    test_qmarks!(0 "" );

    test_qmarks_with_col!("bla","bli"; "bla=$1,bli=$2");
    test_qmarks_with_col!("bla"; "bla=$1");
    test_qmarks_with_col!(; "");
}
