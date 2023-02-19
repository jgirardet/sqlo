use crate::{Maison, PPool, PieceFk};
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
    nb_result!(p,PieceFk, la > la, 6);
    // index as arg
    let array = [0, 1, 2, 3];
    nb_result!(p,PieceFk, lg == array[1], 1);
    nb_result!(p,PieceFk, lg > array[1], 8);
    // field as arg
    let a = A { a: 2 };
    nb_result!(p,PieceFk, lg > a.a, 7);
    // use String
    let adr = "adresse2".to_string();
    nb_result!(p,Maison, adresse == adr, 1);
    // rhs uses field not vs
    #[allow(unused_variables)]
    let taille = 1;
    nb_result!(p,Maison, id<= ::taille, 3);
    nb_result!(p,Maison, id == ::taille, 0);
    nb_result!(p,Maison, id == taille, 1);


}}

Test! {select_test_where_parethesis, async fn func(p: PPool) {
    nb_result!(p,PieceFk, (la > 100 || la < 60) && maison_id == 1, 2);
    nb_result!(p,PieceFk, (la < 100), 9);
    nb_result!(p,PieceFk, !(la < 100), 0);
}}

Test! {select_test_in, async fn func(p: PPool) {

    // In
    nb_result!(p,PieceFk, maison_id..[1, 3], 6);
    nb_result!(p,PieceFk, maison_id..(1, 3), 6);
    nb_result!(p,Maison, lespieces.lg..[1, 2, 13], 1); //et non 2 car distinct
    nb_result!(p,PieceFk, maison_id..(0..2), 4);
    nb_result!(p,PieceFk, maison_id..(1..2), 4);
    nb_result!(p,PieceFk, maison_id..(2..=3), 5);
    nb_result!(p,PieceFk, maison_id..(1..4), 9);
    let (d, e, f) = (1, 2, 4);
    nb_result!(p,PieceFk, maison_id..(d, e, f), 7);
    let [d, e, f] = [1, 2, 4];
    nb_result!(p,PieceFk, maison_id..[d, e, f], 7);
}}

Test! {select_test_foreign_key, async fn func(p: PPool) {
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
    nb_result!(p,Maison[c].lespieces, 4);
    nb_result!(p,Maison[1].lespieces, lg > 2, 2);
    nb_result!(p,Maison[c].lespieces, lg >= 1 && la < 90, 3);
    nb_result!(p,Maison[a.a].lespieces, 3); //a=2
    nb_result!(p,Maison[array[2]].lespieces, 3); //=2
    nb_result!(p, Maison, taille>100 && lespieces.lg >8, 2);
}}
