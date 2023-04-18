use crate::{Lit, Maison, PPool, PieceFk, PieceFk2};
use sqlo::select;

Test! {select_with_pk, async fn func(p: PPool) {
    // --------------------- select easy -----------------------//

    // pk
    let res = select![.Maison where id == 1](&p.pool)
        .await
        .unwrap();
    assert_eq!(res.id, 1);
    assert_eq!(res.adresse, "adresse1");
    assert_eq!(res.taille, 101);
    // only pk
    let res2 = select![.Maison[1]](&p.pool).await.unwrap();
    assert_eq!(res, res2);
}}

Test! {select_with_attribute, async fn func(p: PPool) {
    // one attribute
    let res = select![*Maison where taille > 101]
        (&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].id, 2);
    assert_eq!(res[2].id, 4);
}}

macro_rules! nb_result {
        ($p:ident, $ident:expr, $res:literal) => {
            assert_eq!(
                select![* $ident]
                    (&$p.pool)
                    .await
                    .unwrap()
                    .len(),
                $res
            );
        };
        ($p:ident, $ident:expr, $exp:expr, $res:literal) => {
            assert_eq!(
                select![* $ident where $exp]
                    (&$p.pool)
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
    nb_result!(p,Maison, 4);
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
    let res = select![*Maison where id == -id + 2*id](&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
    let res = select![ * Maison where taille == 100 - -  id](&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
    let res = select![*Maison where id == -id * (-id - -id) + id](&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
    // not
    let res = select![*Maison where !(id == 1)](&p.pool).await.unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].id, 2);
    assert_eq!(res[2].id, 4);
    let res = select![ *Maison  where !(id!=1) && !(id==1 && id==taille)](&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
}}

Test! {select_test_where_null, async fn func(p: PPool) {
    // IS NULL/ IS NOT NULL
    nb_result!(p,Maison, piscine == None, 4);
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
    let res = select![*PieceFk where la > ::la](&p.pool).await.unwrap();
    assert_eq!(res.len(), 6);
    // index as arg
    let array = [0, 1, 2, 3];
    let res = select![*PieceFk where lg == ::array[1]](&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    nb_result!(p,PieceFk, lg == ::array[1], 1);
    nb_result!(p,PieceFk, lg > ::array[1], 8);
    // // field as arg
    let a = A { a: 2 };
    nb_result!(p,PieceFk, lg > ::a.a, 7);
    // // use String
    let adr = "adresse3".to_string();
    nb_result!(p,Maison, adresse == ::adr, 1);
    // // rhs uses field not vs
    #[allow(unused_variables)]
    let taille = 1;
    nb_result!(p,Maison, id<= taille, 4);
    nb_result!(p,Maison, id == ::taille, 1);
    nb_result!(p,Maison, id == taille, 0);
    // use variable since no field exist
    #[allow(unused_variables)]
    let bla = 1;
    nb_result!(p, Maison, id==bla, 1);

    // long patha
    mod moda {
        pub const A:i32= 1;
    }
    nb_result!(p,Maison, id == ::moda::A, 1);


}}

Test! {select_test_where_parethesis, async fn func(p: PPool) {
    let res = select![*PieceFk where (la > 100 || la < 60) && maison_id == 1](&p.pool).await.unwrap();
    assert_eq!(res.len(), 2);
    let res = select![*PieceFk where (la < 100)](&p.pool).await.unwrap();
    assert_eq!(res.len(), 9);
}}

Test! {select_test_wherein, async fn func(p: PPool) {
    let res = select![*PieceFk where maison_id in (1,3)](&p.pool).await.unwrap();
    assert_eq!(res.len(), 6);
    let [d, e, f] = [1, 2, 4];
    let res = select![*PieceFk where maison_id in (::d, ::e, ::f)](&p.pool).await.unwrap();
    assert_eq!(res.len(), 7);
}}

Test! {select_test_where_like, async fn func(p: PPool) {
    let res = select![*Maison where adresse # "adr%"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 3);
    let res = select![*Maison where adresse # "%dress%"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 4);
    let res = select![*Maison where adresse # "%dresse1%"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = select![*Maison where adresse # "%dresse1"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = select![*Maison where adresse # "a%se1"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = select![*Maison where adresse # "a%se1"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
    // // with fk
    let res = select![*Maison where adres.rue # "a%se1"](&p.pool).await.unwrap();
    assert_eq!(res.len(), 1);
}}

Test! {select_test_where_foreign_key, async fn func(p: PPool) {
    let a = A { a: 2 };
    let array = [0, 1, 2, 3];
    // ForeignKey
    let res = select![*Maison[1].lespieces](&p.pool).await.unwrap();
    assert_eq!(res.len(), 4);
    let res: Vec<PieceFk> = select![*Maison[1].lespieces]
        (&p.pool)
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
    let res = select![.Maison where trim(adresse) == "adresse2"](&p.pool).await.unwrap();
    assert_eq!(res.adresse, "   adresse2    ");
    // call n column and where, use of context::Call
    let res = select![.Maison replace(adresse, "3", "XX") as "adresse?:String" where upper(adresse) == "ADRESSE3"](&p.pool).await.unwrap();
    assert_eq!(res.adresse, Some("adresseXX".into()));
    // with fk
    let res = select![.Maison where trim(adres.rue) == "adresse2"](&p.pool).await.unwrap();
    assert_eq!(res.adresse, "   adresse2    ");
}}

Test! {select_cutoms_fields, async fn func(p: PPool) {
  // field
  let  res = select![*Maison id](&p.pool).await.unwrap();
  assert_eq!(res.len(), 4);
  let mut res = res.iter().map(|x|x.id).collect::<Vec<_>>();
  res.sort();
  assert_eq!(res,vec![1,2,3,4]);
  //
  let res = select![*PieceFk lg](&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // two fields
  let res = select![*PieceFk lg, la](&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // with where close
  let res = select![*PieceFk lg where lg >=6](&p.pool).await.unwrap();
  assert_eq!(res.len(), 4);
}}

Test! {select_cutoms_fields_related_join, async fn func(p: PPool) {
  // with related
  let res = select![*Maison[1].lespieces lg](&p.pool).await.unwrap();
  assert_eq!(res.len(), 4);
  // with related and where
  let res = select![*Maison[1].lespieces lg where lg> 2](&p.pool).await.unwrap();
  assert_eq!(res.len(), 2);
  // with join alone
  let res = select![*Maison lespieces.lg](&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // with join and where
  let res = select![*Maison lespieces.lg where lespieces.lg > 2](&p.pool).await.unwrap();
  assert_eq!(res.len(), 7);
  // plain struct join
  let res = select![*Maison where adres.id >2](&p.pool).await.unwrap();
  assert_eq!(res[0].id, 3);

  // with join and where alias
//   let res = select![*Maison upper(id,id) as bla where bla > 2](&p.pool).await.unwrap();
//   assert_eq!(res.len(), 7);
  // left join field
  let res = select![*Maison id, lespieces.lg](&p.pool).await.unwrap();
  assert_eq!(res.len(),9);
  let res = select![*Maison id, lespieces=.lg as "lg?"](&p.pool).await.unwrap();
  assert_eq!(res.len(),10);
  assert_eq!(res[9].lg, None);
  //Left join sqlite works without `?`
  #[cfg(feature="sqlite")]
  let res = select![*Maison id, lespieces=.lg as lg](&p.pool).await.unwrap();
  #[cfg(feature="sqlite")]
  assert_eq!(res.len(),10);
  //left join where
  let res = select![*Maison id,  lespieces=.lg as lespieces? where lespieces=.lg == None](&p.pool).await.unwrap();
  assert_eq!(res.len(), 1);
  let res = select![*Maison id,  lespieces=.lg as lep? where lespieces=.lg == None](&p.pool).await.unwrap();
  assert_eq!(res.len(), 1);
  // self join
    let res = select![*SelfRelation name, manager.name as manager order_by name](&p.pool).await.unwrap();
    assert_eq!(res.len(),2);
    assert_eq!(res[0].name, "axel");
    assert_eq!(res[0].manager, "papa");
    let res = select![*SelfRelation name!, manager=.name as manager?](&p.pool).await.unwrap();
    assert_eq!(res.len(),3);

}}

Test! {select_cutoms_cast, async fn func(p: PPool) {
  //with cast
  let res = select![*Maison adresse as lid](&p.pool).await.unwrap();
  assert!(res[0].lid.contains("adresse"));
  // with join in cast
  let res = select![*Maison lespieces.lg as lll](&p.pool).await.unwrap();
  assert_eq!(res.len(), 9);
  // alias string
  use uuid::Uuid;
  select![*PieceFk nb as "nb:Uuid"](&p.pool).await.unwrap();
  // alias force non null
  select![*PieceFk nb as "nb!"](&p.pool).await.unwrap();
  // alias force nullable
  select![*PieceFk nb as "nb?"](&p.pool).await.unwrap();
  // alias force cast nullable
  select![*PieceFk lg as "nb?:i16"](&p.pool).await.unwrap();
  // alias force cast non  nullable
  select![*PieceFk lg as "nb!:i16"](&p.pool).await.unwrap();
  // long path
  let res = select![*PieceFk nb as "nb:Uuid"](&p.pool).await.unwrap();
  let res2 = select![*PieceFk nb as "nb:uuid::Uuid"](&p.pool).await.unwrap();
  assert_eq!(res[0].nb, res2[0].nb);
  // Non nullable without string
  let res  = select![*PieceFk lg as p!, la as a?](&p.pool).await.unwrap();
  assert![res[0].p >0 ];
  assert![res[0].a.is_some()];
  // Non nullable without alias
  let res = select![*SelfRelation manager_id! where manager_id != None order_by -manager_id](&p.pool).await.unwrap();
  assert_eq!(res[0].manager_id, 3);
  //  nullable without alias
  let res = select![*SelfRelation id?](&p.pool).await.unwrap();
  assert!(res[0].id.is_some());
}}

Test! {select_cutoms_join_conflict, async fn func(p: PPool) {
  // with join conflict column
  let res = select![*Maison id as idm, adres.id where adres.id>"1"](&p.pool).await.unwrap();
  assert_eq!(res[0].idm, 2);
  assert_eq!(res[1].idm, 3);
  assert_eq!(res[0].id, "2");
  assert_eq!(res[1].id, "3");
  // with join conflict column, the reverse with id
  let res = select![*Maison id, adres.id as ll where adres.id>"1"](&p.pool).await.unwrap();
  assert_eq!(res.len(), 2);
}}

Test! {select_customs_call, async fn func(p: PPool) {
  let res = select![.Maison count(id) as total!](&p.pool).await.unwrap();
  assert_eq!(res.total, 4);
  // call with literal
  let res = select![.Maison replace(adresse, "1", "345") as "adr!:String" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.adr, "adresse345");
  // call with literal int
  let res = select![.Maison abs(-taille) as "abs!:i32" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.abs, 101);
  // call with rust variable
  let a = -345.4;
  let res = select![.Maison abs(::a) as "abs!:f64" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.abs, 345.4);
  // call with rust index
  let a = [5.1,6.3,8.0];
  let res = select![.Maison abs(::a[1]) as "abs!:f64" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.abs, 6.3);
  // call with rust field
  struct A{a:f64}
  let a = A{a:6.5};
  let res = select![.Maison abs(::a.a) as "abs!:f64" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.abs, 6.5);
  // call with rust variable outside function
  let aaa = "1";
  #[cfg(not(feature="postgres"))]
  let res = sqlx::query![r#"SELECT  ? as "bla!:String" FROM maison a"#, aaa].fetch_one(&p.pool).await.unwrap();
  #[cfg(feature="postgres")]
  let res = sqlx::query![r#"SELECT  $1 as "bla!:String" FROM maison a"#, aaa].fetch_one(&p.pool).await.unwrap();
  assert_eq!(res.bla, "1");
  // works in sqlite without &str
  #[cfg(feature="sqlite")]
  let aaa = 1;
  #[cfg(feature="sqlite")]
  let res = select![.Maison ::aaa as "bla!:i16"](&p.pool).await.unwrap();
  #[cfg(feature="sqlite")]
  assert_eq!(res.bla, 1);
}}

Test! {select_customs_binary_operation, async fn func(p: PPool) {
    // binary
  let res = select![.Maison id + 3 as "id_plus_3!:i32" where id == 2](&p.pool).await.unwrap();
  assert_eq!(res.id_plus_3, 5);
  // complex binary
  let a = 22;
  let res = select![.Maison ::a + id - abs(5.0)  as "total:f64" where id==2](&p.pool).await.unwrap();
  assert_eq!(res.total, Some(19.0)); //2 + 22  -5
  // test mul et div
  let res = select![.Maison id * id as "total!"  where id==1](&p.pool).await.unwrap();
  assert_eq!(res.total, 1);
  let res = select![.Maison id / id as "total!"  where id==1](&p.pool).await.unwrap();
  #[cfg(not(feature="mysql"))]
  assert_eq!(res.total, 1);
  #[cfg(feature="mysql")]
  assert_eq!(res.total, 1.into()); //uses bigdecimal
  // test all equlity ops
  let res = select![.Maison id<2 as "total!:bool" where id ==1 ](&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![.Maison id==2 as "total!:bool" where id==2 ](&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![.Maison 2>id as "total!:bool" where id ==1](&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![.Maison 2>=id as "total!:bool" where id ==1 ](&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![.Maison id==id as "total!:bool" where id==1 ](&p.pool).await.unwrap();
  assert!(res.total);
  let res = select![.Maison id!=2 as "total!:bool"  where id==1](&p.pool).await.unwrap();
  assert!(res.total);
  // op inside call
  #[cfg(not(feature="mysql"))]
  {let res = select![.Maison sign(sign(-id)) as "c!:f64" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.c, -1.0);}
  #[cfg(feature="mysql")]
  {let res = select![.Maison sign(sign(-id)) as "c!" where id==1](&p.pool).await.unwrap();
  assert_eq!(res.c, -1);}
//   // binary asterisk
  let res = select![.Maison id * 3 as "id_plus_3:i32" where id==2](&p.pool).await.unwrap();
  assert_eq!(res.id_plus_3, Some(6));
}}

Test! {select_customs_asterisk, async fn func(p: PPool) {
    let res = select![.Maison count(*) as total!](&p.pool).await.unwrap();
    assert_eq!(res.total, 4);
}}

struct B {
    bb: i64,
}

#[derive(Debug, PartialEq)]
struct Hav {
    total: i64,
    maison_id: i32,
}

Test! {select_customs_struct_custom_with_query_as, async fn func(p: PPool) {

    #[cfg(not(feature="sqlite"))]
    {let res = select![. B, Maison count(*) as "bb!"](&p.pool).await.unwrap();
    assert_eq!(res.bb, 4);}
    #[cfg(feature="sqlite")]
    {let res = select![. B, Maison count(*) as "bb:i64"](&p.pool).await.unwrap();
    assert_eq!(res.bb, 4);}
}}

Test! {select_order_by, async fn func(p:PPool) {
    //simpl asc
   let res  = select![*Lit order_by surface](&p.pool).await.unwrap();
   assert_eq!(res[0].id,2 );
   assert_eq!(res[3].id,3 );
   //smpl desc
   let res  = select![*Lit order_by -surface](&p.pool).await.unwrap();
   assert_eq!(res[0].id,3 );
   assert_eq!(res[3].id,2 );
   // two order by
   let res = select![*Lit order_by surface, -id](&p.pool).await.unwrap();
   assert_eq!(res[0].id, 2);
   assert_eq!(res[1].id, 4);
   assert_eq!(res[2].id, 1);
   assert_eq!(res[3].id, 3);
   // two asc
   let res = select![*Lit order_by surface, id](&p.pool).await.unwrap();
   assert_eq!(res[1].id, 1);
   assert_eq!(res[2].id, 4);
   // two desc
   let res = select![*Lit order_by -surface, -id](&p.pool).await.unwrap();
   assert_eq!(res[0].id, 3);
   assert_eq!(res[1].id, 4);
   assert_eq!(res[2].id, 1);
   assert_eq!(res[3].id, 2);
   // fk simple
   let res = select![*Maison[1].lespieces order_by -lg](&p.pool).await.unwrap();
   assert_eq!(res[0].lg, 9);
   // fk custom
//    let res = select![.Maison count(lespieces.lg) as total order_by -total](&p.pool).await.unwrap(); à fixer ??
   let res = select![.Maison count(lespieces.lg) as total! order_by -total](&p.pool).await.unwrap();
   assert_eq!(res.total, 9);
   // with where + fk
   let res = select![*Maison where lespieces.lg > 7  order_by -taille](&p.pool).await.unwrap();
   assert_eq!(res.len(), 2);
   assert_eq!(res[0].id, 2);
   assert_eq!(res[1].id, 1);
   // square_bracket_syntax
   let res = select![*Lit order_by[surface, id]](&p.pool).await.unwrap();
   assert_eq!(res[1].id, 1);
   assert_eq!(res[2].id, 4);
   // with string alias and non string alias used
   let res = select![*Lit surface as "ss!" order_by surface](&p.pool).await.unwrap();
   assert_eq!(res[0].ss, 100);
}}

Test! {column_alias_in_clauses, async fn func(p:PPool) {
    // where alias
    select![ .Maison taille! where taille>1](&p.pool).await.unwrap();
    // where full alias
    select![ .Maison taille as "taille!:i32" where taille>1](&p.pool).await.unwrap();
    // where callable
    select![ .Maison abs(taille) as "total:i32" where total>1](&p.pool).await.unwrap();
    // where callable args
    select![.Maison abs(1.0) as "total:i64" where total>=1.0](&p.pool).await.unwrap();
    // // order by alias
    select![ .Maison taille! order_by taille](&p.pool).await.unwrap();
    // order_by full alias
    select![ .Maison taille as "taille!:i32" order_by taille](&p.pool).await.unwrap();
    // order_by callable
    select![ .Maison abs(taille) as total! order_by total](&p.pool).await.unwrap();
    // order_by and where
    select![.Maison taille! where taille > 1 order_by taille](&p.pool).await.unwrap();
    // order_by and where
    select![.Maison taille! where taille > 1 order_by taille](&p.pool).await.unwrap();
    // order_by and where callable
    select![.Maison abs(taille) as "total!:i32" where total>1 order_by total](&p.pool).await.unwrap();
    // callable
   select![ .Maison count(lespieces.lg) as total order_by -total](&p.pool).await.unwrap();
   select![ .PieceFk maison_id, count(lg) as "total!:i32" group_by maison_id order_by total](&p.pool).await.unwrap();
}}

Test! {select_limit, async fn func(p:PPool) {
    // simple
    let res = select![*PieceFk  limit 2,3](&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].lg, 4];
    assert_eq![res[1].lg, 5];
    // with order by : force class types
    use uuid::Uuid;
    let res = select![*PieceFk,PieceFk nb as "nb!:Uuid", lg as "lg!:i32", la as "la!", maison_id as "maison_id!" order_by -lg limit 2,3](&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].lg, 6];
    assert_eq![res[1].lg, 5];
    // with order by, force class  type, bracket
    let res = select![*Maison, Maison id as "id!", taille as "taille!", adresse as "adresse!", piscine as "piscine?:bool"
    order_by[-taille] limit[2,1]](&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].id, 3];
    assert_eq![res[1].id, 2];
    // page simple
    let res = select![*PieceFk limit 2,4](&p.pool).await.unwrap(); //4;5
    let res2 = select![*PieceFk page 3,2](&p.pool).await.unwrap();
    assert_eq!(res, res2);
}}

Test! {select_group_by, async fn func(p:PPool) {
    // simple
    let res = select![*PieceFk maison_id, count(*) as "total?:i64" where maison_id==1 group_by maison_id ](&p.pool).await.unwrap();
    assert_eq!(res[0].total, Some(4));
    // with fk
    let res = select![*Maison lespieces.maison_id, count(*) as "total?:i64" where id==1 group_by lespieces.maison_id](&p.pool).await.unwrap();
    assert_eq!(res[0].total, Some(4));
    // with order by with "full qualified aggregate"
    let res = select![*PieceFk maison_id, count(*) as "total?:i64" where maison_id==3 group_by maison_id order_by count(*)](&p.pool).await.unwrap();
    assert_eq!(res[0].total, Some(2));
    // with order by with "with alias"
    let res = select![*PieceFk maison_id, count(*) as "total?:i64"  where maison_id==3 group_by maison_id order_by count(*)](&p.pool).await.unwrap();
    assert_eq!(res[0].total, Some(2));
    // with order by with string alias
    let res = select![*PieceFk maison_id, count(*) as "total?:i64" where maison_id==3 group_by maison_id order_by count(*)](&p.pool).await.unwrap();
    assert_eq!(res[0].total, Some(2));
}}

Test! {select_having, async fn func(p:PPool){
    // standard
    let res = select![ * Hav,  PieceFk maison_id as "maison_id!:_"  , count(lg) as "total!:_" group_by maison_id having total>=3 order_by maison_id](&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].maison_id, 1];
    assert_eq![res[0].total, 4];
    assert_eq![res[1].maison_id, 2];
    assert_eq![res[1].total, 3];
    // fk
    let res2 = sqlo::select![*Hav,Maison id as "maison_id:_", count(lespieces.lg) as "total!:_" group_by id having total>=3 order_by maison_id](&p.pool).await.unwrap();
    let res3 = sqlx::query_as!(Hav, r#"SELECT DISTINCT maison.id as "maison_id:_", count(piece.lg)  as "total!:_" FROM maison INNER JOIN piece ON maison.id=piece.maison_id GROUP BY maison.id HAVING count(piece.lg) >= 3 ORDER BY maison.id"#)
    .fetch_all(&p.pool).await.unwrap();
    assert_eq![res, res2];
    assert_eq![res, res3];
    // // // two conditions
    let res = select![*Hav,PieceFk maison_id as "maison_id:_", count(lg) as "total!:_" group_by maison_id having total >3 && total < 5](&p.pool).await.unwrap();
    assert_eq![res.len(), 1];
    assert_eq![res[0].maison_id, 1];
    assert_eq![res[0].total, 4];
}}

Test! {select_sub_select, async fn func(p:PPool){
    // simple
    let res = select![*Maison where id == {PieceFk count(*) where lg==9 limit 1}](&p.pool).await.unwrap();
    assert_eq![res.len(), 1];
    assert_eq![res[0].id, 1];
    // use comparator
    let res = select![*PieceFk where maison_id == {Maison id where taille == 101 limit 1}](&p.pool).await.unwrap();
    assert_eq![res.len(), 4];
    // use in
    let res = select![*PieceFk where maison_id in {Maison id where taille > 101}](&p.pool).await.unwrap();
    assert_eq![res.len(), 5];
    // use fk
    let res = select![*Lit where id in {Maison lespieces.maison_id where lespieces.lg>=8}](&p.pool).await.unwrap();
    assert_eq![res.len(), 2];
    assert_eq![res[0].id, 1];
    assert_eq![res[1].id, 2];
    // in column
    let res = select![*Lit id, surface, {PieceFk count(*)  where maison_id==Lit.id} as p!](&p.pool).await.unwrap();
    assert_eq![res.len(),4];
    assert_eq![res[0].id,1];
    assert_eq![res[0].p,4];
    assert_eq![res[3].id,4];
    assert_eq![res[3].p,0];
}}

Test! {select_sub_select_exists, async fn func(p:PPool){
    // exists
    let res = select![*Maison where exists {PieceFk lg where maison_id==4}](&p.pool).await.unwrap();
    assert_eq!(res.len(), 0);
    let res = select![*Maison where exists {PieceFk lg where lg == Maison.id}](&p.pool).await.unwrap();
    assert_eq!(res.len(), 4);
}}

Test! {select_case, async fn func(p:PPool){
    //simple with case
    let res = select![*Maison id, match id 1=>"un" as "a?:String" order_by id](&p.pool).await.unwrap();
    assert_eq!(res[0].a, Some("un".to_string()));
    //simple with case and _
    let res = select![*Maison id, match id 1=>"un",_=>"lol" as "a!:String" order_by id](&p.pool).await.unwrap();
    assert_eq!(res[0].a, "un");
    assert_eq!(res[0].id, 1);
    assert_eq!(res[1].a, "lol");
    assert_eq!(res[1].id, 2);
    assert_eq!(res[2].a, "lol");
    // more
    let res = select![*Maison id, match id 1=>"un",2=>"deux",_=>"lol" as "a!:String" order_by id](&p.pool).await.unwrap();
    assert_eq!(res[0].a, "un");
    assert_eq!(res[0].id, 1);
    assert_eq!(res[1].a, "deux");
    assert_eq!(res[1].id, 2);
    assert_eq!(res[2].a, "lol");

    //simple no case
    let res = select![*Maison id, match id<2=>"un" as "a:String" order_by id](&p.pool).await.unwrap();
    assert_eq!(res[0].a, Some("un".to_string()));
    //simple with case and _
    let res = select![*Maison id, match id<2=>"un",_=>"lol" as "a!:String" order_by id](&p.pool).await.unwrap();
    assert_eq!(res[0].a, "un");
    assert_eq!(res[0].id, 1);
    assert_eq!(res[1].a, "lol");
    assert_eq!(res[1].id, 2);
    assert_eq!(res[2].a, "lol");
}}

Test! {select_optional_and_stream, async fn func(p:PPool){
   use futures_lite::stream::StreamExt;
   let mut stream = select![+ Maison](&p.pool);
   for i in 1..=4 {
       assert_eq!(stream.try_next().await.unwrap().unwrap().id,i )
   }
   assert!(stream.try_next().await.unwrap().is_none());

   // optional
   let res = select![? Maison where id==99999](&p.pool).await.unwrap();
   assert_eq![res, None];
}}
