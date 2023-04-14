use crate::{Maison, PPool, PieceFk, PieceFk2};
use sqlo::insert;

Test! {insert_simple, async fn func(p: PPool) {
    //all field with one Some
    insert!( Maison id=5, adresse="lieu5", taille=23, piscine= true)(&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 5).await.unwrap(), Maison{id:5, adresse:"lieu5".to_string(), taille: 23, piscine:Some(true)});

    // auto pk;
    insert!(Maison adresse="lieu6", taille=23, piscine= true)(&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 6).await.unwrap(), Maison{id:6, adresse:"lieu6".to_string(), taille: 23, piscine:Some(true)});

    // None implicit
    insert!( Maison id=7, adresse="lieu7", taille=23)(&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 7).await.unwrap(), Maison{id:7, adresse:"lieu7".to_string(), taille: 23, piscine: None});

    // None explicit
    insert!( Maison id=8, adresse="lieu8", taille=23, piscine=None)(&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, 8).await.unwrap(), Maison{id:8, adresse:"lieu8".to_string(), taille: 23, piscine: None});

   // pk Uuid
   let ua = uu4!(A);
   insert![PieceFk nb=ua, lg=1, la= 53, maison_id=8](&p.pool).await.unwrap();
   assert_eq!(PieceFk::get(&p.pool, &ua).await.unwrap(), PieceFk{nb:ua, lg:1, la:53, maison_id:8});

   // variable without ::
   let ua = 9;
    insert!( Maison id=ua, adresse="lieu9", taille=23, piscine=None)(&p.pool).await.unwrap();
    assert_eq!(Maison::get(&p.pool, ua).await.unwrap(), Maison{id:ua, adresse:"lieu9".to_string(), taille: 23, piscine: None});


}}

Test! {insert_returning, async fn func(p: PPool) {
    // one
    let maison = insert!(. Maison id=5, adresse="lieu5", taille=23, piscine= true)(&p.pool).await.unwrap();
    assert_eq!(maison, Maison{id:5, adresse:"lieu5".to_string(), taille: 23, piscine:Some(true)});

    // many
    let maison = insert!(* Maison id=6, adresse="lieu6", taille=23, piscine= true)(&p.pool).await.unwrap();
    assert_eq!(maison[0], Maison{id:6, adresse:"lieu6".to_string(), taille: 23, piscine:Some(true)});
}}

Test! {insert_with_create_fn, async fn func(p: PPool) {
   // fetch_one
   let res = insert![. PieceFk2 lg=2, la= 53, maison_id=1](&p.pool).await.unwrap();
   assert_eq!(PieceFk2::get(&p.pool, &res.nb).await.unwrap(), res);
   // execute
   insert![PieceFk2 lg=9999, la=11111, maison_id=1](&p.pool).await.unwrap();
   assert_eq!(sqlo::select![.PieceFk2 where la==11111](&p.pool).await.unwrap().lg, 9999);
   // same args
   insert![PieceFk2 lg=1, la=2222, maison_id=1](&p.pool).await.unwrap();
   assert_eq!(sqlo::select![.PieceFk2 where la==2222](&p.pool).await.unwrap().lg, 1);
}}
