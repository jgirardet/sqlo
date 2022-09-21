#[derive(sqlo::Sqlo)]
struct Maison {
    id: i64,
    adresse: String,
    taille: i64,
    piscine: Option<bool>,
}

#[async_std::main]
async fn main() {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let m = Maison::get(&pool, 1).await.unwrap();
    set_Maison![&pool, m, taille = 3].await.unwrap();
    m;
}
