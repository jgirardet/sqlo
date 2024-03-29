// CREATE TABLE maison (
//   id INTEGER NOT NULL PRIMARY KEY,
//   adresse TEXT NOT NULL,
//   taille INTEGER NOT NULL,
//   piscine BOOLEAN
// );

// CREATE TABLE piece (
//   id UUID NOT NULL PRIMARY KEY,
//   lg INTEGER NOT NULL,
//   la INTEGER NOT NULL,
//   maison_id INTEGER NOT NULL,
//   FOREIGN KEY(maison_id) REFERENCES maison(id)
// );

use sqlx::{Connection, SqliteConnection};
use uuid::Uuid;

macro_rules! uu4 {
    (1) => {
        uuid::uuid!("11111111111111111111111111111111")
    };
}

// without any attr
#[derive(sqlo::Sqlo, Debug, PartialEq)]
struct Maison {
    // #[sqlo(type_override)]
    id: i64,
    adresse: String,
    taille: i64,
    piscine: Option<bool>,
}

// with a single attr in sqlo attr
#[derive(sqlo::Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "piece")]
struct WithAttrs {
    #[sqlo(primary_key, type_override, insert_fn = "uuid::Uuid::new_v4")]
    nb: Uuid,
    #[sqlo(type_override, column = "lg")]
    lgogog: i32,
    la: i64,
    maison_id: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "maison")]
struct Maison2 {
    #[sqlo(type_override)]
    id: i32,
    adresse: String,
    taille: i64,
    piscine: Option<bool>,
}

#[async_std::main]
async fn main() {
    let mut conn = SqliteConnection::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    //test utils as instance
    let m = Maison::get(&mut conn, 1).await.unwrap();
    assert_eq!(m.sqlo_struct_name(), "Maison",);

    //test get
    let m = Maison::get(&mut conn, 1).await.unwrap();
    assert_eq!(m.adresse, "adresse1", "test get method");

    // test primary_key attirbute
    let p = WithAttrs::get(&mut conn, &uu4!(1)).await.unwrap(); //

    // test type_override
    let m2 = Maison2::get(&mut conn, 1).await.unwrap(); //typeoverride
    assert_eq!(m2.id, 1i32, "type_override converte i32 instead of i64");

    // test column rename
    assert_eq!(
        p,
        WithAttrs {
            nb: uu4!(1),
            lgogog: 1,
            la: 10,
            maison_id: 1
        }
    );

    //test save from new instance not in database
    let mut sn = Maison {
        id: 123,
        adresse: "zef".to_string(),
        taille: 234,
        piscine: Some(true),
    };
    sn.save(&mut conn).await.unwrap();
    let snn = Maison::get(&mut conn, 123).await.unwrap();
    assert_eq!(sn, snn);

    //test save  with instance already saved
    sn.adresse = "AA".to_string();
    sn.save(&mut conn).await.unwrap();
    assert_eq!(
        Maison::get(&mut conn, 123).await.unwrap().adresse,
        "AA".to_string()
    );
}
