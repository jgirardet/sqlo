#![allow(dead_code)]
use sqlo::Sqlo;

#[derive(Sqlo, Debug, PartialEq)]
struct Maison {
    #[sqlo(type_override, create_fn = "uuid::Uuid::new_v4")]
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
    let a = Maison {
        id: 123,
        adresse: "zefzef".to_string(),
        taille: 4,
        piscine: None,
    };
    a.save(&pool).await.unwrap();
    let b = set_Maison![&pool, a, taille = 1].await.unwrap();
    assert_eq!(b.taille, 1);
    assert_eq!(b.id, 123);
    let c = Maison::get(&pool, 123).await.unwrap();
    assert_eq!(b, c);
}

#[test]
fn test_main_expand() {
    main();
}
