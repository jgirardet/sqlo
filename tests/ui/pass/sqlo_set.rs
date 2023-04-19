#![deny(unused_macros)]
use sqlo::Sqlo;
use sqlx::{Connection, SqliteConnection};

#[derive(Debug, PartialEq, Sqlo)]
struct Maison {
    // #[sqlo(type_override)]
    id: i64,
    adresse: String,
    taille: i64,
    piscine: Option<bool>,
}

#[async_std::main]
async fn main() {
    let mut conn = SqliteConnection::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let pool = &mut conn;

    let _m = Maison::get(pool, 1).await.unwrap();
    // sqlo_set!(m; taille=23);
}
