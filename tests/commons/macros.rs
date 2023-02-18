#[rustfmt::skip]
macro_rules! uu4 {
    (1) => {uuid::uuid!("11111111111111111111111111111111")};
    (8) => {uuid::uuid!("88888888888888888888888888888888")};
    (9) => {uuid::uuid!("99999999999999999999999999999999")};
}

macro_rules! test_base {
    ($name:ident, $test_fn:item, $backend:ident) => {
        paste::paste! {

            #[allow(non_snake_case)]
            #[sqlx::test(migrations = "tests/migrations")]
            async fn [<$name _ $backend>](pool: sqlx::[<$backend Pool>]) {
                $test_fn
                let pol = $crate::PPool{pool};
                func(pol).await
            }
        }
    };
}

#[cfg(feature = "sqlite")]
macro_rules! Test {
    ($name: ident, $test_fn:item) => {
        test_base! {$name, $test_fn, Sqlite}
    };
}
