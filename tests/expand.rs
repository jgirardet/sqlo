#![allow(dead_code)]

use sqlo::Sqlo;


// use sqlx::Connection;
// use uuid::Uuid;
// // #[derive(Debug, sqlo::Sqlo, PartialEq)]

#[derive(Sqlo)]
struct Maison {
    // #[sqlo(type_override, create_fn = "uuid::Uuid::new_v4")]
    id: i64, // #[sqlo(column = "content")]
    adresse: String
             // active: bool,
}

// // #[cfg(not(any(features = "sqlite")))]

// #[cfg(features = "sqlite")]
// type DB = sqlx::SqliteConnection;

// #[cfg(features = "default")]
// type DB = sqlx::AnyConnection;

// async fn get_pool() -> DB {
//     DB::connect(&std::env::var("DATABASE_URL").unwrap())
//         .await
//         .unwrap()
// }

// #[async_std::test]
// async fn main() {
//     // let _t = thisoneok {
//     //     id: "bla".to_string(),
//     //     value: 23,
//     // };

//     println!("SANS QLITE");
//     let pool = get_pool().await;
//     dbg!(&pool);

//     #[cfg(feature = "sqlite")]
//     println!("This is sqlite ");
//     // let pool = &mut conn;

//     // let l = IdUniqueInt::get(pool, 2);

//     // for (k, v) in std::env::vars() {
//     //     println!("{k}:{v}");
//     // }
//     // assert_eq!(Maison::tablename(), "maison");
//     // let l = MatcherEntries::create(&mut conn, "bla".to_string(), false)
//     //     .await
//     //     .unwrap();

//     // set_MatcherEntries!(&mut conn, l, contenu = "zegergergerg", active = false);
//     // assert_eq!(l.structname(), "MatcherEntries".to_string());
//     // sqlo_set!["matcher_entries" l; content="zefzef".to_string(), active=true];
//     // let m = MatcherEntries::get(&mut conn, l.id).await.unwrap();
//     // assert_eq!(l, m);
//     // assert_eq!(WithAttrs::tablename(), "piece");
//     // assert_eq!(WithTwoAttrs::tablename(), "maison");
// }
