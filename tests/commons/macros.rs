#[rustfmt::skip]
macro_rules! uu4 {
    (1) => {uuid::uuid!("11111111111111111111111111111111")};
    (8) => {uuid::uuid!("88888888888888888888888888888888")};
    (9) => {uuid::uuid!("99999999999999999999999999999999")};
}

macro_rules! run_test_base {
    ($categorie:ident, $test_fn:ident, $backend:ident) => {
        paste::paste! {

            #[sqlx::test(migrations = "tests/migrations")]
            async fn [<$categorie _ $test_fn>](pool: sqlx::[<$backend Pool>]) {
                crate::$test_fn(crate::PPool { pool }).await;
            }
        }
    };
}

#[cfg(feature = "sqlite")]
macro_rules! run_test {
    ($categorie:ident: $test_fn:ident) => {
        run_test_base!($categorie, $test_fn, Sqlite);
    };
}

#[cfg(feature = "sqlite")]
macro_rules! run_many {
    ($categorie:ident: $($fns:ident),+) => {
        $(run_test!($categorie: $fns);)+
    };
}
