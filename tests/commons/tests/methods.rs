use crate::{Adresse, Maison, Maison2, PPool, WithAttrs};
Test! {pseudo_hidden_methods, async fn func(p: PPool) {
    //test tablname
    //test utils as instance
    let m = Maison::get(&p.pool, 1).await.unwrap();
    assert_eq!(m.pk(), &1);
    assert_eq!(m.sqlo_struct_name(), "Maison");
}}

Test! {get_and_attribute, async fn func(p: PPool) {
    let m = Maison::get(&p.pool, 1).await.unwrap();
    assert_eq!(m.adresse, "adresse1", "test get method");

    // test primary_key attirbute
    let pr = WithAttrs::get(&p.pool, &uu4!(1)).await.unwrap(); //

    // test type_override
    let m2 = Maison2::get(&p.pool, 1).await.unwrap(); //typeoverride
    assert_eq!(m2.id, 1i32, "type_override converte i32 instead of i64");

    // test column rename
    assert_eq!(
        pr,
        WithAttrs {
            nb: uu4!(1),
            lglg: 1,
            la: 10,
            maison_id: 1
        }
    );

    // test get with pk as String and string literal
    let a = Adresse::get(&p.pool, "1").await.unwrap();
    let b = Adresse::get(&p.pool, "1").await.unwrap();
    assert_eq!(a, b);
}}

Test! {save, async fn func(p: PPool) {
    //test save from new instance not in database
    let mut sn = Maison {
        id: 123,
        adresse: "zef".to_string(),
        taille: 234,
        piscine: Some(true),
    };
    sn.save(&p.pool).await.unwrap();
    let snn = Maison::get(&p.pool, 123).await.unwrap();
    assert_eq!(sn, snn);

    //test save  with instance already saved
    sn.adresse = "AA".to_string();
    sn.save(&p.pool).await.unwrap();
    assert_eq!(
        Maison::get(&p.pool, 123).await.unwrap().adresse,
        "AA".to_string()
    );
}}

Test! {delete, async fn func(p: PPool) {
    // delete by instance objet
    let u9 = uu4!(9);
    let neuf = WithAttrs::get(&p.pool, &u9).await.unwrap();
    neuf.remove(&p.pool).await.unwrap();
    assert!(WithAttrs::get(&p.pool, &u9).await.is_err());

    // delete by pk object Copy
    let u8 = uu4!(8);
    WithAttrs::get(&p.pool, &u8).await.unwrap();
    WithAttrs::delete(&p.pool, &u8).await.unwrap();
    assert!(WithAttrs::get(&p.pool, &u8).await.is_err());

    // delete by pk String non Copy
    let m  = Adresse::get(&p.pool, "1").await.unwrap();
    Adresse::delete(&p.pool, &m.id).await.unwrap();
    assert!(Adresse::get(&p.pool, &m.id).await.is_err());

    // delete by pk int
    let m  = Maison::get(&p.pool, 4).await.unwrap();
    Maison::delete(&p.pool, m.id).await.unwrap();
    assert!(Maison::get(&p.pool, m.id).await.is_err());
}}
