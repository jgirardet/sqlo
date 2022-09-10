use sqlx::Connection;

#[derive(Debug, sqlo::Sqlo)]
struct IdUniqueUuid {
    #[sqlo(create_fn = "uuid::Uuid::new_v4")]
    id: uuid::Uuid,
}

#[derive(Debug, sqlo::Sqlo, PartialEq)]
#[sqlo(tablename = "id_unique_int")]
struct IdUniqueIntCreateArg {
    #[sqlo(create_arg)]
    id: i64,
}

#[derive(Debug, sqlo::Sqlo, PartialEq)]
#[sqlo(tablename = "id_unique_int")]
struct IdUniqueIntAuto {
    id: i64,
}

fn main() {
    let mut conn = sqlx::SqliteConnection::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
}
