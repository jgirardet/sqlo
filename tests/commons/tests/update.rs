use crate::{Adresse, Maison, PPool, WithAttrs};

pub async fn update(p: PPool) {
    let sn = Maison::get(&p.pool, 1).await.unwrap();

    // test update one field with same name / column
    let sn = update_Maison![ sn ;  adresse = "BB"](&p.pool)
        .await
        .unwrap();
    assert_eq!(
        Maison::get(&p.pool, 1).await.unwrap().adresse,
        "BB".to_string()
    );

    // test update more than one field with same name / column
    update_Maison![ sn ;  adresse = "CC", taille = 999](&p.pool)
        .await
        .unwrap();
    assert_eq!(
        Maison::get(&p.pool, 1).await.unwrap().adresse,
        "CC".to_string()
    );
    assert_eq!(Maison::get(&p.pool, 1).await.unwrap().taille, 999);

    // test updaet with different column/field name
    let pr = WithAttrs::get(&p.pool, &uu4!(1)).await.unwrap(); //
    let q = update_WithAttrs![pr ;  lglg = 123789](&p.pool)
        .await
        .unwrap();
    let r = WithAttrs::get(&p.pool, &uu4!(1)).await.unwrap(); //
    assert_eq!(q.lglg, 123789);
    assert_eq!(q, r);

    // test update with update like instance syntax
    let r = update_WithAttrs![r ; lglg = 23](&p.pool).await.unwrap();
    assert_eq!(r.lglg, 23);

    //
    // test update with various primarykey format for the instance
    //

    // simple variable
    let r = update_WithAttrs![ r ; lglg = 45](&p.pool).await.unwrap();
    assert_eq!(r.lglg, 45);

    // literal string
    let a = update_Adresse![pk "1" ; rue = "fzefzef"](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("fzefzef".to_string()));

    // literal int
    let m = update_Maison![pk 1 ; taille=567](&p.pool).await.unwrap();
    assert_eq!(m.taille, 567);

    // by variable
    let lavar = String::from("1");
    let a = update_Adresse![pk lavar ; rue = "gg"](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("gg".to_string()));

    // by index
    let v = vec!["1", "2"];
    let a = update_Adresse!(pk v[0] ; rue = "rr")(&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("rr".to_string()));

    // by field
    #[derive(Debug)]
    struct B {
        b: String,
    }
    let c = B { b: "1".to_string() };
    let a = update_Adresse!(pk c.b ; rue = "rr")(&p.pool).await.unwrap();
    assert_eq!(a.rue, Some("rr".to_string()));

    // by field_nested
    #[derive(Debug)]
    struct D {
        d: B,
    }
    let lad = D {
        d: B { b: "1".to_string() },
    };
    let a = update_Adresse!(pk lad.d.b; rue ="oo")(&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("oo".to_string()));

    // by var as ref
    let lavar = String::from("1");
    let value = "aze".to_string();
    let reflav = &lavar;
    let revalue = &value;
    let a = update_Adresse![pk reflav ; rue = revalue](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("aze".to_string()));
}
