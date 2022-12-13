use crate::{Maison, PPool, PieceFk};
use sqlo::sqlo_select;

pub async fn sqlo_select(p: PPool) {
    // --------------------- select easy -----------------------//

    // pk
    let res = sqlo_select!(Maison where id == 1)
        .fetch_one(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.id, 1);
    assert_eq!(res.adresse, "adresse1");
    assert_eq!(res.taille, 101);

    // one attribute
    let res = sqlo_select!(Maison where taille > 101)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].id, 2);
    assert_eq!(res[1].id, 3);

    macro_rules! comp_many {
        ($ident:expr, $res:literal) => {
            assert_eq!(
                sqlo_select!($ident)
                    .fetch_all(&p.pool)
                    .await
                    .unwrap()
                    .len(),
                $res
            );
        };
        ($ident:expr, $exp:expr, $res:literal) => {
            assert_eq!(
                sqlo_select!($ident where $exp)
                    .fetch_all(&p.pool)
                    .await
                    .unwrap()
                    .len(),
                $res
            );
        }
    }

    //empty where
    comp_many!(Maison, 3);
    // standard expressions - use literal as arg
    comp_many!(PieceFk, la == 30, 1);
    comp_many!(PieceFk, la != 30, 8);
    comp_many!(PieceFk, la > 30, 6);
    comp_many!(PieceFk, la >= 30, 7);
    comp_many!(PieceFk, la < 30, 2);
    comp_many!(PieceFk, la <= 30, 3);
    // IS NULL/ IS NOT NULL
    comp_many!(Maison, piscine == None, 3);
    comp_many!(Maison, piscine != None, 0);
    // between
    comp_many!(PieceFk, la <= 30 && la < 60, 3);
    comp_many!(PieceFk, la <= 30 && la > 30 || la == 50, 1);
    let la = 34;
    // ident/variable as arg
    comp_many!(PieceFk, la > la, 6);
    // index as arg
    let array = [0, 1, 2, 3];
    comp_many!(PieceFk, lg == array[1], 1);
    comp_many!(PieceFk, lg > array[1], 8);
    // field as arg
    struct A {
        a: i32,
    }
    let a = A { a: 2 };
    comp_many!(PieceFk, lg > a.a, 7);
    // use String
    let adr = "adresse2".to_string();
    comp_many!(Maison, adresse == adr, 1);
    // Parenthesis
    comp_many!(PieceFk, (la > 100 || la < 60) && maison_id == 1, 2);
    comp_many!(PieceFk, (la < 100), 9);
    comp_many!(PieceFk, !(la < 100), 0);

    // Disctint
    comp_many!(Maison, lespieces.la > 10, 3);

    // ForeignKey
    comp_many!(Maison[1].lespieces, 4);
    let res: Vec<PieceFk> = sqlo_select!(Maison[1].lespieces)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[3].la, 90);

    let c = 1;
    comp_many!(Maison[c].lespieces, 4);
    comp_many!(Maison[1].lespieces, lg > 2, 2);
    comp_many!(Maison[c].lespieces, lg >= 1 && la < 90, 3);
    comp_many!(Maison[a.a].lespieces, 3); //a=2
    comp_many!(Maison[array[2]].lespieces, 3); //=2

    // In
    comp_many!(PieceFk, maison_id..[1, 3], 6);
    comp_many!(PieceFk, maison_id..(1, 3), 6);
    comp_many!(Maison, lespieces.lg..[1, 2, 13], 1); //et non 2 car distinct
    comp_many!(PieceFk, maison_id..(0..2), 4);
    comp_many!(PieceFk, maison_id..(1..2), 4);
    comp_many!(PieceFk, maison_id..(2..=3), 5);
    comp_many!(PieceFk, maison_id..(1..4), 9);
    let (d, e, f) = (1, 2, 4);
    comp_many!(PieceFk, maison_id..(d, e, f), 7);
    let [d, e, f] = [1, 2, 4];
    comp_many!(PieceFk, maison_id..[d, e, f], 7);
}
