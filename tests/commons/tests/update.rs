use crate::{Adresse, Maison, PPool, WithAttrs};
use sqlo::{select, update};

Test! {update_all_rows, async fn func(p: PPool) {
    // simple all rows
    update![Maison adresse = "all"].execute(&p.pool).await.unwrap();
    let res = select![Maison].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].adresse, "all");
    assert_eq!(res[1].adresse, "all");
    assert_eq!(res[2].adresse, "all");
}}

Test! {update_pk, async fn func(p: PPool) {
// simple one field
update![Maison[1] adresse = "BB"].execute(&p.pool).await.unwrap();
assert_eq!(
    Maison::get(&p.pool, 1).await.unwrap().adresse,
    "BB".to_string()
);
// update has been done only for 1 not for every rows
assert_eq!(
    Maison::get(&p.pool, 2).await.unwrap().adresse,
    "adresse2".to_string()
);
// Update many columns
update![Maison[1] adresse = "CC", taille = 999].execute(&p.pool)
    .await
    .unwrap();
let res  = Maison::get(&p.pool, 1).await.unwrap();
assert_eq!(
    res.adresse,
    "CC"
    );
assert_eq!(res.taille, 999);
}}

Test! {update_instance, async fn func(p: PPool) {
    let r = Maison::get(&p.pool, 1).await.unwrap();
//     // test update with update like instance syntax
//
    // let r = update![Maison(r) taille = 23](&p.pool).await.unwrap();
    // assert_eq!(r.taille, 23);
    // assert_eq!(r, Maison::get(&p.pool,1));

//     //
//     // test update with various primarykey format for the instance
//     //

//     // simple variable
//     let r = update_WithAttrs![ r ; lglg = 45](&p.pool).await.unwrap();
//     assert_eq!(r.lglg, 45);

//     // literal string
//     let a = update_Adresse![pk "1" ; rue = "fzefzef"](&p.pool)
//         .await
//         .unwrap();
//     assert_eq!(a.rue, Some("fzefzef".to_string()));

//     // literal int
//     let m = update_Maison![pk 1 ; taille=567](&p.pool).await.unwrap();
//     assert_eq!(m.taille, 567);

//     // by variable
//     let lavar = String::from("1");
//     let a = update_Adresse![pk lavar ; rue = "gg"](&p.pool)
//         .await
//         .unwrap();
//     assert_eq!(a.rue, Some("gg".to_string()));

//     // by index
//     let v = vec!["1", "2"];
//     let a = update_Adresse!(pk v[0] ; rue = "rr")(&p.pool)
//         .await
//         .unwrap();
//     assert_eq!(a.rue, Some("rr".to_string()));

//     // by field
//     #[derive(Debug)]
//     struct B {
//         b: String,
//     }
//     let c = B { b: "1".to_string() };
//     let a = update_Adresse!(pk c.b ; rue = "rr")(&p.pool).await.unwrap();
//     assert_eq!(a.rue, Some("rr".to_string()));

//     // by field_nested
//     #[derive(Debug)]
//     struct D {
//         d: B,
//     }
//     let lad = D {
//         d: B { b: "1".to_string() },
//     };
//     let a = update_Adresse!(pk lad.d.b; rue ="oo")(&p.pool)
//         .await
//         .unwrap();
//     assert_eq!(a.rue, Some("oo".to_string()));

//     // by var as ref
//     let lavar = String::from("1");
//     let value = "aze".to_string();
//     let reflav = &lavar;
//     let revalue = &value;
//     let a = update_Adresse![pk reflav ; rue = revalue](&p.pool)
//         .await
//         .unwrap();
//     assert_eq!(a.rue, Some("aze".to_string()));
}}
