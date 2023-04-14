#[rustfmt::skip]
macro_rules! uu4 {
    (1) => {uuid::uuid!("11111111111111111111111111111111")};
    (2) => {uuid::uuid!("22222222222222222222222222222222")};
    (3) => {uuid::uuid!("33333333333333333333333333333333")};
    (5) => {uuid::uuid!("55555555555555555555555555555555")};
    (6) => {uuid::uuid!("66666666666666666666666666666666")};
    (8) => {uuid::uuid!("88888888888888888888888888888888")};
    (9) => {uuid::uuid!("99999999999999999999999999999999")};
    (A) => {uuid::uuid!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")};
}

macro_rules! test_base {
    ($name:ident, $test_fn:item, $backend:ident, $db_path:literal) => {
        paste::paste! {

            #[allow(non_snake_case)]
            #[sqlx::test(migrations = "tests/migrations/" $db_path)]
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
        test_base! {$name, $test_fn, Sqlite, "sqlite"}
    };
}

#[cfg(feature = "postgres")]
macro_rules! Test {
    ($name: ident, $test_fn:item) => {
        test_base! {$name, $test_fn, Pg, "pg"}
    };
}
