#![allow(dead_code)]

use sqlo::Sqlo;

// #[derive(Sqlo, Debug, PartialEq)]
// struct Maison {
//     #[sqlo(type_override, create_fn = "uuid::Uuid::new_v4")]
//     id: i64,
//     adresse: String,
//     taille: i64,
//     piscine: Option<bool>,
// }

// #[derive(sqlo::Sqlo)]
// struct Adresse {
//     id: String,
//     rue: Option<String>,
// }

#[derive(Debug, Sqlo)]
struct IdUniqueUuid {
    #[sqlo(create_fn = "uuid::Uuid::new_v4")]
    id: uuid::Uuid,
}
#[async_std::main]
async fn main() {
    let _pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    // let a = Maison {
    //     id: 123,
    //     adresse: "zefzef".to_string(),
    //     taille: 4,
    //     piscine: None,
    // };
    // a.save(&pool).await.unwrap();

    // let s = "aa".to_string();
    // let a = update_Maison!(a; adresse = "zefzef")(&pool).await.unwrap();
    // assert_eq!(&a.adresse, "zefzef");
    // let a = update_Maison!(a; adresse=s)(&pool).await.unwrap();
    // assert_eq!(&a.adresse, "aa");
    // let a = Adresse {
    //     id: "1".to_string(),
    //     rue: None,
    // };
    // let aa = Adresse {
    //     id: "1".to_string(),
    //     rue: None,
    // };
    // struct B {
    //     c: Adresse,
    // }
    // let b = B { c: a };
    // let a = Some("1".to_string());
    // let e = Some("1".to_string());
    // let var = "1".to_string();
    // let b = &a;
    //
    // let v = &var;
    // update_Adresse![pk v; rue = e](&pool).await.unwrap();
    // dbg!(&var);
    // let g = set_Adresse![for aa do rue = e](&pool).await.unwrap();
    // assert_eq!(f, g);

    // let m1 = Maison::get(&pool, 1).await.unwrap();
    // let m2 = set_Maison![for m1 do taille=2](&pool).await.unwrap();
    // let a = set_Adresse![where "12" do rue = d](&pool).await.unwrap();
    // assert_eq!(a.rue.as_ref().unwrap(), "1");
    // assert_eq!(b.taille, 1);
    // assert_eq!(b.id, 123);

    // // let uu = uuid::Uuid::new_v4();
    // // let us = uu.to_string();
    // let c = Maison::get(&pool, 123).await.unwrap();
    // let d = set_Maison![&pool, 123, taille = 1].await.unwrap();
    // assert_eq!(b, c);
}

#[test]
fn test_main_expand() {
    main();
}
