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

#[rustfmt::skip]
macro_rules! uu4 {
    (1) => {uuid::uuid!("11111111111111111111111111111111")};
    (8) => {uuid::uuid!("88888888888888888888888888888888")};
    (9) => {uuid::uuid!("99999999999999999999999999999999")};
}

// without any attr
#[derive(sqlo::Sqlo, Debug, PartialEq)]
struct Maison {
    id: i64,
    adresse: String,
    taille: i64,
    piscine: Option<bool>,
}

// with a single attr in sqlo attr
#[derive(sqlo::Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "piece")]
struct WithAttrs {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    nb: uuid::Uuid, // keep full path
    #[sqlo(type_override, column = "lg")]
    lglg: i32,
    la: i64,
    maison_id: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "maison")]
struct Maison2 {
    #[sqlo(create_arg, type_override)]
    id: i32,
    adresse: String,
    taille: i64,
    piscine: Option<bool>,
}

#[derive(sqlo::Sqlo, PartialEq, Debug)]
struct Adresse {
    #[sqlo(create_arg)]
    id: String,
    rue: Option<String>,
}

#[async_std::main]
async fn main() {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    //  ------------------- Test additional utils ------------------------------//

    //test tablname
    assert_eq!(
        Maison::tablename(),
        "maison",
        "tablename is derived from struct name"
    );
    assert_eq!(
        WithAttrs::tablename(),
        "piece",
        "attribute tablename force override tablename"
    );

    //test utils as instance
    let m = Maison::get(&pool, 1).await.unwrap();
    assert_eq!(m.itablename(), "maison");

    //  ------------------- Test get and Attribute ------------------------------//

    //test get
    let m = Maison::get(&pool, 1).await.unwrap();
    assert_eq!(m.adresse, "adresse1", "test get method");

    // test primary_key attirbute
    let p = WithAttrs::get(&pool, &uu4!(1)).await.unwrap(); //

    // test type_override
    let m2 = Maison2::get(&pool, 1).await.unwrap(); //typeoverride
    assert_eq!(m2.id, 1i32, "type_override converte i32 instead of i64");

    // test column rename
    assert_eq!(
        p,
        WithAttrs {
            nb: uu4!(1),
            lglg: 1,
            la: 10,
            maison_id: 1
        }
    );

    // test get with pk as String and string literal
    let a = Adresse::get(&pool, &"1".to_string()).await.unwrap();
    let b = Adresse::get(&pool, "1").await.unwrap();
    assert_eq!(a, b);

    //  ------------------- Test create  ------------------------------//

    // test create : primary_key auto generated by db
    // test create returning option and non option
    let nm = Maison::create(&pool, "labas", 23, None).await.unwrap();
    assert_eq!(nm.adresse, "labas".to_string());
    let nmm = Maison::get(&pool, nm.id).await.unwrap();
    assert_eq!(nm, nmm);

    // test create with `create` attribute
    let nm2 = Maison2::create(&pool, 999, "le999", 9, Some(true))
        .await
        .unwrap();
    assert_eq!(nm2.id, 999i32);

    // test create with `create_fn` attribute
    let wa = WithAttrs::create(&pool, 123, 12, 1).await.unwrap();
    wa.nb.as_hyphenated(); //test uuid type by ducj typing

    //  ------------------- Test save ------------------------------//

    //test save from new instance not in database
    let mut sn = Maison {
        id: 123,
        adresse: "zef".to_string(),
        taille: 234,
        piscine: Some(true),
    };
    sn.save(&pool).await.unwrap();
    let snn = Maison::get(&pool, 123).await.unwrap();
    assert_eq!(sn, snn);

    //test save  with instance already saved
    sn.adresse = "AA".to_string();
    sn.save(&pool).await.unwrap();
    assert_eq!(
        Maison::get(&pool, 123).await.unwrap().adresse,
        "AA".to_string()
    );

    // --------------------- Test delete -----------------------------------------//

    // delete by instance objet
    let u9 = uu4!(9);
    let neuf = WithAttrs::get(&pool, &u9).await.unwrap();
    neuf.remove(&pool).await.unwrap();
    assert!(WithAttrs::get(&pool, &u9).await.is_err());

    // delete by pk object Copy
    let u8 = uu4!(8);
    WithAttrs::get(&pool, &u8).await.unwrap();
    WithAttrs::delete(&pool, &u8).await.unwrap();
    assert!(WithAttrs::get(&pool, &u8).await.is_err());

    // delete by pk int
    let m = Adresse::create(&pool, "deleteme", None).await.unwrap();
    Adresse::delete(&pool, &m.id).await.unwrap();
    assert!(Adresse::get(&pool, &m.id).await.is_err());

    // delete by pk String non Copy
    let m = Maison::create(&pool, "zfef", 23, None).await.unwrap();
    Maison::delete(&pool, m.id).await.unwrap();
    assert!(Maison::get(&pool, m.id).await.is_err());
    //  ------------------- Test sqlo_set // update ------------------------------//

    // test update one field with same name / column
    let sn = update_Maison![ sn ;  adresse = "BB"](&pool).await.unwrap();
    assert_eq!(
        Maison::get(&pool, 123).await.unwrap().adresse,
        "BB".to_string()
    );

    // test update more than one field with same name / column
    update_Maison![ sn ;  adresse = "CC", taille = 999](&pool)
        .await
        .unwrap();
    assert_eq!(
        Maison::get(&pool, 123).await.unwrap().adresse,
        "CC".to_string()
    );
    assert_eq!(Maison::get(&pool, 123).await.unwrap().taille, 999);

    // test updaet with different column/field name
    let p = WithAttrs::get(&pool, &uu4!(1)).await.unwrap(); //
    let q = update_WithAttrs![p ;  lglg = 123789](&pool).await.unwrap();
    let r = WithAttrs::get(&pool, &uu4!(1)).await.unwrap(); //
    assert_eq!(q.lglg, 123789);
    assert_eq!(q, r);

    // test update with update like instance syntax
    let r = update_WithAttrs![r ; lglg = 23](&pool).await.unwrap();
    assert_eq!(r.lglg, 23);

    //
    // test update with various primarykey format for the instance
    //

    // simple variable
    let r = update_WithAttrs![ r ; lglg = 45](&pool).await.unwrap();
    assert_eq!(r.lglg, 45);

    // literal string
    let a = update_Adresse![pk "1" ; rue = "fzefzef"](&pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("fzefzef".to_string()));

    // literal int
    let m = update_Maison![pk 123 ; taille=567](&pool).await.unwrap();
    assert_eq!(m.taille, 567);

    // by variable
    let lavar = String::from("1");
    let a = update_Adresse![pk lavar ; rue = "gg"](&pool).await.unwrap();
    assert_eq!(a.rue, Some("gg".to_string()));

    // by index
    let v = vec!["1", "2"];
    let a = update_Adresse!(pk v[0] ; rue = "rr")(&pool).await.unwrap();
    assert_eq!(a.rue, Some("rr".to_string()));

    // by field
    #[derive(Debug)]
    struct B {
        b: String,
    }
    let c = B { b: "1".to_string() };
    let a = update_Adresse!(pk c.b ; rue = "rr")(&pool).await.unwrap();
    assert_eq!(a.rue, Some("rr".to_string()));

    // by field_nested
    #[derive(Debug)]
    struct D {
        d: B,
    }
    let lad = D {
        d: B { b: "1".to_string() },
    };
    let a = update_Adresse!(pk lad.d.b; rue ="oo")(&pool).await.unwrap();
    assert_eq!(a.rue, Some("oo".to_string()));

    // by var as ref
    let lavar = String::from("1");
    let value = "aze".to_string();
    let reflav = &lavar;
    let revalue = &value;
    let a = update_Adresse![pk reflav ; rue = revalue](&pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("aze".to_string()));

    // ----------------- End -----------------------------------//
    println!("Sqlite Maison succeds !!!")
}
