use crate::{Adresse, Maison, PPool, WithAttrs};
use sqlo::{select, update};

Test! {update_all_rows, async fn func(p: PPool) {
    // simple all rows
    update![Maison adresse = "all"](&p.pool).await.unwrap();
    let res = select![Maison].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].adresse, "all");
    assert_eq!(res[1].adresse, "all");
    assert_eq!(res[2].adresse, "all");
}}

Test! {update_pk, async fn func(p: PPool) {
// simple one field
update![Maison[1] adresse = "BB"](&p.pool).await.unwrap();
assert_eq!(
    Maison::get(&p.pool, 1).await.unwrap().adresse,
    "BB".to_string()
);
// update has been done only for 1 not for every rows
assert_eq!(
    Maison::get(&p.pool, 2).await.unwrap().adresse, "adresse2");
// Update many columns
update![Maison[1] adresse = "CC", taille = 999](&p.pool)
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
    // test update with update like instance syntax
    update![Maison(r) taille = 23](&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 1).await.unwrap().taille, 23);
}}

Test! {update_returning_pk, async fn func(p: PPool) {
    // test update with update like instance syntax
   let t  = update![. Maison[1] taille = 53](&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 1).await.unwrap().taille, 53);
    assert_eq!(t.taille, 53);
    // with uuid
    // let pk = uu4!{uu3!{1};
    let r  = update![. WithAttrs[uu4!{1}] lglg = 53](&p.pool).await.unwrap();
    assert_eq!(WithAttrs::get(&p.pool, &uu4!{1}).await.unwrap().lglg, 53);
    assert_eq!(r.lglg, 53);
}}

Test! {update_returning_instance, async fn func(p: PPool) {
    let r = Maison::get(&p.pool, 1).await.unwrap();
    // test update with update like instance syntax
    let t  = update![. Maison(r) taille = 53](&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 1).await.unwrap().taille, 53);
    assert_eq!(t.taille, 53);

}}

Test! {update_pk_various_types, async fn func(p: PPool) {
    // test update with various primarykey format for the instance
    // simple variable
    let a = 45;
    let r = update![. WithAttrs  lglg = ::a](&p.pool).await.unwrap();
    assert_eq!(r.lglg, 45);

    // literal string
    let a = update![. Adresse["1"]  rue = "fzefzef"](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("fzefzef".to_string()));

    // literal int
    let m = update![.Maison[1] taille=567](&p.pool).await.unwrap();
    assert_eq!(m.taille, 567);

    // by variable
    let lavar = String::from("1");
    let a = update![.Adresse[lavar] rue = "gg"](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("gg".to_string()));

    // by index
    let v = vec!["1", "2"];
    let a = update![. Adresse[v[0]]  rue = "rr"](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("rr".to_string()));

    // by field
    #[derive(Debug)]
    struct B {
        b: String,
    }
    let c = B { b: "1".to_string() };
    let a = update!(. Adresse[c.b] rue = "rr")(&p.pool).await.unwrap();
    assert_eq!(a.rue, Some("rr".to_string()));

    // by field_nested
    #[derive(Debug)]
    struct D {
        d: B,
    }
    let lad = D {
        d: B { b: "1".to_string() },
    };
    let a = update!(. Adresse[lad.d.b] rue ="oo")(&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("oo".to_string()));

    // by var as ref
    let lavar = String::from("1");
    let value = "aze".to_string();
    let reflav = &lavar;
    let revalue = &value;
    let a = update![.Adresse[reflav]  rue = ::revalue](&p.pool)
        .await
        .unwrap();
    assert_eq!(a.rue, Some("aze".to_string()));

}}

Test! {update_pk_fetch_mode, async fn func(p: PPool) {
   // fetch
   use futures_lite::stream::StreamExt;
   let mut stream = update![+ Maison taille=22](&p.pool);
   for _ in 0..4 {
       assert_eq!(stream.try_next().await.unwrap().unwrap().taille,22 )
   }
   assert!(stream.try_next().await.unwrap().is_none());

   // many
   let res = update![* Maison taille=32](&p.pool).await.unwrap();
   assert_eq![res.len(), 4];
   assert_eq![res[0].taille, 32];
   assert_eq![res[3].taille, 32];

   // optional
   let res = update![? Maison taille=42](&p.pool).await.unwrap().unwrap(); //second unwroap for option
   assert_eq![res.id, 1];
   assert_eq![res.taille, 42];


   // fetch_one, execute already tested higher
}}
