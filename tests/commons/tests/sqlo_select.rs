use crate::{Lit, Maison, PPool, PieceFk, PieceFk2};
use sqlo::select;

Test! {select_with_pk, async fn func(p: PPool) {
    // --------------------- select easy -----------------------//

    // pk
    let res = select!(Maison where id == 1)
        .fetch_one(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.id, 1);
    assert_eq!(res.adresse, "adresse1");
    assert_eq!(res.taille, 101);
}}

Test! {select_with_attribute, async fn func(p: PPool) {
    // one attribute
    let res = select!(Maison where taille > 101)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].id, 2);
    assert_eq!(res[1].id, 3);
}}

macro_rules! nb_result {
        ($p:ident, $ident:expr, $res:literal) => {
            assert_eq!(
                select!($ident)
                    .fetch_all(&$p.pool)
                    .await
                    .unwrap()
                    .len(),
                $res
            );
        };
        ($p:ident, $ident:expr, $exp:expr, $res:literal) => {
            assert_eq!(
                select!($ident where $exp)
                    .fetch_all(&$p.pool)
                    .await
                    .unwrap()
                    .len(),
                $res
            );
        }
    }

Test! {select_test_always_disctinct, async fn func(p: PPool) {
    nb_result!(p,Maison, lespieces.la > 10, 3);
}}

Test! {select_test_where_binary, async fn func(p: PPool) {
    //empty where
    nb_result!(p,Maison, 3);
    // standard expressions - use literal as arg
    nb_result!(p,PieceFk, la == 30, 1);
    nb_result!(p,PieceFk, la != 30, 8);
    nb_result!(p,PieceFk, la > 30, 6);
    nb_result!(p,PieceFk, la >= 30, 7);
    nb_result!(p,PieceFk, la < 30, 2);
    nb_result!(p,PieceFk, la <= 30, 3);

}}
Test! {select_test_where_unary, async fn func(p: PPool) {
    // minus
    let res = select![Maison where id == -1 + 2].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
    let res = select![Maison where id == 1 - -1].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].id, 2);
    let res = select![Maison where id == -1 * (-2 - -1)].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
    // not
    let res = select![Maison where !(id == 1)].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].id, 2);
    assert_eq!(res[1].id, 3);
    let res = select![Maison  where !(id!=1) && !(1==1 && 0==1)].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
}}

Test! {select_test_where_null, async fn func(p: PPool) {
    // IS NULL/ IS NOT NULL
    nb_result!(p,Maison, piscine == None, 3);
    nb_result!(p,Maison, piscine != None, 0);
}}

Test! {select_test_where_between, async fn func(p: PPool) {
    nb_result!(p,PieceFk, la <= 30 && la < 60, 3);
    nb_result!(p,PieceFk, la <= 30 && la > 30 || la == 50, 1);
}}

struct A {
    a: i32,
}

Test! {select_test_where_rust_var_as_arg, async fn func(p: PPool) {
    // ident/variable as arg
    let la = 34;
    let res = select![PieceFk where la > ::la].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 6);
    // index as arg
    let array = [0, 1, 2, 3];
    let res = select![PieceFk where lg == ::array[1]].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    nb_result!(p,PieceFk, lg == ::array[1], 1);
    nb_result!(p,PieceFk, lg > ::array[1], 8);
    // // field as arg
    let a = A { a: 2 };
    nb_result!(p,PieceFk, lg > ::a.a, 7);
    // // use String
    let adr = "adresse2".to_string();
    nb_result!(p,Maison, adresse == ::adr, 1);
    // // rhs uses field not vs
    #[allow(unused_variables)]
    let taille = 1;
    nb_result!(p,Maison, id<= taille, 3);
    nb_result!(p,Maison, id == ::taille, 1);
    nb_result!(p,Maison, id == taille, 0);
    // long patha
    mod moda {
        pub const A:i32= 1;
    }
    nb_result!(p,Maison, id == ::moda::A, 1);
}}

Test! {select_test_where_parethesis, async fn func(p: PPool) {
    let res = select![PieceFk where (la > 100 || la < 60) && maison_id == 1].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 2);
    let res = select![PieceFk where (la < 100)].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 9);
}}

Test! {select_test_wherein, async fn func(p: PPool) {
    let res = select![PieceFk where maison_id in (1,3)].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 6);
    let [d, e, f] = [1, 2, 4];
    let res = select!(PieceFk where maison_id in (::d, ::e, ::f)).fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 7);
}}

Test! {select_test_where_like, async fn func(p: PPool) {
    let res = select![Maison where adresse # "adr%"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 3);
    let res = select![Maison where adresse # "%dress%"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 3);
    let res = select![Maison where adresse # "%dresse1%"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = select![Maison where adresse # "%dresse1"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = select![Maison where adresse # "a%se1"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = select![Maison where adresse # "a%se1"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    // // with fk
    let res = select![Maison where adres.rue # "a%se1"].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
}}

Test! {select_test_where_foreign_key, async fn func(p: PPool) {
    let a = A { a: 2 };
    let array = [0, 1, 2, 3];
    // ForeignKey
    nb_result!(p,Maison[1].lespieces, 4);
    let res: Vec<PieceFk> = select!(Maison[1].lespieces)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[3].la, 90);

    let c = 1;
    // various pk args
    nb_result!(p,Maison[c].lespieces, 4);
    nb_result!(p,Maison[1].lespieces, lg > 2, 2);
    nb_result!(p,Maison[c].lespieces, lg >= 1 && la < 90, 3);
    nb_result!(p,Maison[a.a].lespieces, 3); //a=2
    nb_result!(p,Maison[array[2]].lespieces, 3); //=2

    // no related name specified
    nb_result!(p,Maison[c].piece_fk2, 4);

    // join in wherre taken in account
    nb_result!(p, Maison, taille>100 && lespieces.lg >=8, 2);
    nb_result!(p, Maison, lespieces.lg>4 && adres.rue == "adresse1", 1);
}}

Test! {select_test_where_call, async fn func(p:PPool){
    // simple
    let res = select![Maison where trim(adresse, "adr") == "esse2"].fetch_one(&p.pool).await.unwrap();
    assert_eq!(res.adresse, "adresse2");
    // call n column and where, use of context::Call
    let res = select![Maison replace(adresse, "2", "XX") as "adresse?:String" where upper(adresse) == "ADRESSE2"].fetch_one(&p.pool).await.unwrap();
    assert_eq!(res.adresse, Some("adresseXX".into()));
    // with fk
    let res = select![Maison where trim(adres.rue, "adr") == "esse2"].fetch_one(&p.pool).await.unwrap();
    assert_eq!(res.adresse, "adresse2");
}}

Test! {select_cutoms_fields, async fn func(p: PPool) {
  // field
  let res = select![Maison id].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 3);
  assert_eq!(res[0].id, 1);
  assert_eq!(res[2].id, 3);
  let res = select![PieceFk lg].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // two fields
  let res = select![PieceFk lg, la].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // with where close
  let res = select![PieceFk lg where lg >=6].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 4);
}}

Test! {select_cutoms_fields_related_join, async fn func(p: PPool) {
  // with related
  let res = select![Maison[1].lespieces lg].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 4);
  // with related and where
  let res = select![Maison[1].lespieces lg where lg> 2].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 2);
  // with join alone
  let res = select![Maison lespieces.lg].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // with join and where
  let res = select![Maison lespieces.lg where lespieces.lg > 2].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 7);
}}

Test! {select_cutoms_cast, async fn func(p: PPool) {
  //with cast
  let res = select![Maison adresse as lid].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res[2].lid, "adresse3");
  // with join in cast
  let res = select![Maison lespieces.lg as lll].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // alias string
  use uuid::Uuid;
  select![PieceFk nb as "nb:Uuid"].fetch_all(&p.pool).await.unwrap();
  // alias force non null
  select![PieceFk nb as "nb!"].fetch_all(&p.pool).await.unwrap();
  // alias force nullable
  select![PieceFk nb as "nb?"].fetch_all(&p.pool).await.unwrap();
  // alias force cast nullable
  select![PieceFk lg as "nb?:i16"].fetch_all(&p.pool).await.unwrap();
  // alias force cast non  nullable
  select![PieceFk lg as "nb!:i16"].fetch_all(&p.pool).await.unwrap();
  // long path
  let res = select![PieceFk nb as "nb:Uuid"].fetch_all(&p.pool).await.unwrap();
  let res2 = select![PieceFk nb as "nb:uuid::Uuid"].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res[0].nb, res2[0].nb);
  // Non nullable without string
  let res  = select![ PieceFk lg as p!, la as a?].fetch_all(&p.pool).await.unwrap();
  assert_eq![res[0].p,1 ];
  assert_eq![res[0].a,Some(10) ];
}}

Test! {select_cutoms_join_conflict, async fn func(p: PPool) {
  // with join conflict column
  let res = select![Maison id as idm, adres.id where adres.id>1].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res[0].idm, 2);
  assert_eq!(res[1].idm, 3);
  assert_eq!(res[0].id, "2");
  assert_eq!(res[1].id, "3");
  // with join conflict column, the reverse with id
  let res = select![Maison id, adres.id as ll where adres.id>1].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res.len(), 2);
}}

Test! {select_cutoms_call, async fn func(p: PPool) {
  let res = select![Maison count(id) as total].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.total, 3);
  // call with literal
  let res = select![Maison replace(adresse, "1", "345") as "adr!:String" where id==1].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.adr, "adresse345");
  // call with literal int
  let res = select![Maison min(id, 2, 45) as "lemin!:u16" where id==1].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.lemin, 1);
  // call with rust variable
  let a = 345;
  let res = select![Maison max(id, 2, ::a) as "lemax!:u16" where id==1].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.lemax, 345);
  // call with rust index
  let a = [5,6,8];
  let res = select![Maison max(id, 2, ::a[1]) as "lemax!:u16" where id==1].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.lemax, 6);
  // call with rust field
  struct A{a:u16}
  let a = A{a:99};
  let res = select![Maison max(id, 2, ::a.a) as "lemax!:u16" where id==1].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.lemax, 99);
  // call with rusdt variable outside function
  let a = 1;
  let res = select![Maison ::a as "bla!:u16"].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.bla, a);
}}

Test! {select_cutoms_binary_operation, async fn func(p: PPool) {
    // binary
  let res = select![Maison id + 3 as id_plus_3].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res[1].id_plus_3, Some(5));
  // complex binary
  let a = 22;
  let res = select![Maison ::a + id - max(3,4,5)  as "total:i16"].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res[1].total, Some(19)); //2 + 22  -5
  // test all arythmetique ops
  let res = select![Maison 1 / 1 * 1 + 1 - 1 as "total!:i16" ].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.total, 1);
  // test all equlity ops
  let res = select![Maison 1<2 as "total!:bool" ].fetch_one(&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![Maison 1<=2 as "total!:bool" ].fetch_one(&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![Maison 2>1 as "total!:bool" ].fetch_one(&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![Maison 2>=1 as "total!:bool" ].fetch_one(&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![Maison 1==1 as "total!:bool" ].fetch_one(&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![Maison 1!=2 as "total!:bool" ].fetch_one(&p.pool).await.unwrap();
  assert!(res.total);
  // op inside call
  let res = select![Maison max(count(id), 5) as "c!:u16"].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.c, 5);
  // binary asterisk
  let res = select![Maison id * 3 as id_plus_3].fetch_all(&p.pool).await.unwrap();
  assert_eq!(res[1].id_plus_3, Some(6));
}}

Test! {select_cutoms_asterisk, async fn func(p: PPool) {
    let res = select![Maison count(*) as total].fetch_one(&p.pool).await.unwrap();
    assert_eq!(res.total, 3);
}}

Test! {select_cutoms__struct_custom_with_query_as, async fn func(p: PPool) {
    let res = select![A, Maison count(*) as a].fetch_one(&p.pool).await.unwrap();
    assert_eq!(res.a, 3);
    let res = select![A, Maison count(*) as "a:i32"].fetch_one(&p.pool).await.unwrap();
    assert_eq!(res.a, 3);
}}

Test! {select_order_by, async fn func(p:PPool) {
    //simpl asc
   let res  = select![Lit order_by surface].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[0].id,2 );
   assert_eq!(res[3].id,3 );
   //smpl desc
   let res  = select![Lit order_by -surface].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[0].id,3 );
   assert_eq!(res[3].id,2 );
   // two order by
   let res = select![Lit order_by surface, -id].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[0].id, 2);
   assert_eq!(res[1].id, 4);
   assert_eq!(res[2].id, 1);
   assert_eq!(res[3].id, 3);
   // two asc
   let res = select![Lit order_by surface, id].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[1].id, 1);
   assert_eq!(res[2].id, 4);
   // two desc
   let res = select![Lit order_by -surface, -id].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[0].id, 3);
   assert_eq!(res[1].id, 4);
   assert_eq!(res[2].id, 1);
   assert_eq!(res[3].id, 2);
   // fk simple
   let res = select![Maison[1].lespieces order_by -lg].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[0].lg, 9);
   // fk custom
   let res = select!(Maison count(lespieces.lg) as total order_by -total).fetch_one(&p.pool).await.unwrap();
   assert_eq!(res.total, 9);
   // with where + fk
   let res = select!(Maison where lespieces.lg > 7  order_by -taille).fetch_all(&p.pool).await.unwrap();
   assert_eq!(res.len(), 2);
   assert_eq!(res[0].id, 2);
   assert_eq!(res[1].id, 1);
   // square_bracket_syntax
   let res = select![Lit order_by[surface, id]].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[1].id, 1);
   assert_eq!(res[2].id, 4);
   // with string alias and non string alias used
   let res = select![Lit surface as "ss!" order_by ss].fetch_all(&p.pool).await.unwrap();
   assert_eq!(res[0].ss, 100);
}}

Test! {select_limit, async fn func(p:PPool) {
    // simple
    let res = select![PieceFk  limit 2,3].fetch_all(&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].lg, 4];
    assert_eq![res[1].lg, 5];
    // with order by : force class types
    use uuid::Uuid;
    let res = select![ PieceFk,PieceFk nb as "nb!:Uuid", lg as "lg!:i32", la as "la!", maison_id as "maison_id!" order_by -lg limit 2,3].fetch_all(&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].lg, 6];
    assert_eq![res[1].lg, 5];
    // with order by, force class  type, bracket
    let res = select![Maison, Maison id as "id!", taille as "taille!", adresse as "adresse!", piscine as "piscine"
    order_by[-taille] limit[2,1]].fetch_all(&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].id, 2];
    assert_eq![res[1].id, 1];
    // page simple
    let res = select![PieceFk limit 2,4].fetch_all(&p.pool).await.unwrap(); //4;5
    let res2 = select![PieceFk page 3,2].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res, res2);
}}

Test! {select_group_by, async fn func(p:PPool) {
    // simple
    let res = select![PieceFk maison_id, count(*) as "total!" group_by maison_id].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].maison_id, 1);
    assert_eq!(res[0].total, 4);
    // with fk
    let res = select![Maison lespieces.maison_id, count(*) as "total!" group_by lespieces.maison_id].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].maison_id, 1);
    assert_eq!(res[0].total, 4);
    // with order by with "full qualified aggregate"
    let res = select![PieceFk maison_id, count(*) as "total!" group_by maison_id order_by count(*)].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].maison_id, 3);
    assert_eq!(res[0].total, 2);
    // with order by with "with alias"
    let res = select![PieceFk maison_id, count(*) as total group_by maison_id order_by total].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].maison_id, 3);
    assert_eq!(res[0].total, Some(2));
    // with order by with string alias
    let res = select![PieceFk maison_id, count(*) as "total!:i32" group_by maison_id order_by total].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].maison_id, 3);
    assert_eq!(res[0].total, 2);
    // with order by with string alias
    let res = select![PieceFk maison_id, count(*) as "total!:i32" group_by maison_id order_by total].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].maison_id, 3);
    assert_eq!(res[0].total, 2);
}}

Test! {select_having, async fn func(p:PPool){
    #[derive(Debug, PartialEq, Eq)]
    struct Hav {
        maison_id: i64,
        total: i32
    }
    // standard
    let res = select![Hav,PieceFk maison_id, sum(lg) as "total!:_" group_by maison_id having total>11].fetch_all(&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].maison_id, 1];
    assert_eq![res[0].total, 18];
    assert_eq![res[1].maison_id, 2];
    assert_eq![res[1].total, 16];
    // fk
    let res2 = sqlo::select![Hav,Maison id as maison_id, sum(lespieces.lg) as "total!:_" group_by id having total>11].fetch_all(&p.pool).await.unwrap();
    let res3 = sqlx::query_as!(Hav, r#"SELECT DISTINCT maison.id  as maison_id, sum(piece.lg)  as "total!:_" FROM maison INNER JOIN piece ON maison.id=piece.maison_id GROUP BY maison.id HAVING "total!:_" > ?"#, 11)
    .fetch_all(&p.pool).await.unwrap();
    assert_eq![res, res2];
    assert_eq![res, res3];
    // two conditions
    let res = select![Hav,PieceFk maison_id, sum(lg) as "total!:_" group_by maison_id having total>11 && total <18].fetch_all(&p.pool).await.unwrap();
    assert_eq![res.len(), 1];
    assert_eq![res[0].maison_id, 2];
    assert_eq![res[0].total, 16];
}}
