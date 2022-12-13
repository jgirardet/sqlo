#[sqlx::test(migrations = "tests/migrations")]
async fn bla(pool: sqlx::SqlitePool) {
    let res = sqlo::sqlo![SELECT COUNT(id) AS b FROM Maison]
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(res.b, 3);
}
