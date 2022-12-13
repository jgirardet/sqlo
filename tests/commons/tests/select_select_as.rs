use sqlo::{select, select_as};

use crate::{Maison, PPool};

pub async fn select_macro(p: PPool) {
    // ----------------select------------------
    // #simple
    let res = select![id FROM Maison].fetch_all(&p.pool).await.unwrap();
    assert_eq!(res[0].id, 1);
    assert_eq!(res[1].id, 2);
    // cast
    let res = select![id AS bla FROM Maison]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].bla, 1);
    let res = select![id AS "bla:u8" FROM Maison]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].bla, 1u8);
}

pub async fn select_as_macro(p: PPool) {
    // --------------select_as-----------------
    // #simple
    let res = select_as![Maison, id,adresse, taille, piscine FROM Maison]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 3);

    // # some columns with alias
    struct Bli {
        adresse: String,
        tail: i64,
    }
    let res = select_as![Bli, adresse, taille AS tail FROM Maison]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].adresse, "adresse1");
    assert_eq!(res[1].tail, 102);

    // Distinct
    #[allow(dead_code)]
    struct MaisonId {
        maison_id: i64,
    }
    let res = select_as![MaisonId, maison_id FROM PieceFk]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 9);
    let res = select_as![MaisonId, DISTINCT maison_id FROM PieceFk]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 3);

    // #table alias
    let res = select_as![Maison, id,adresse, taille, m.piscine FROM Maison m]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 3);

    // #two tables alias
    let res = select_as![Bli, b.la AS tail, a.adresse FROM Maison a, PieceFk b ]
        .fetch_one(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.adresse, "adresse1");
    assert_eq!(res.tail, 10);

    // ## Columns ##
    // #function
    struct OptionI32 {
        res: Option<i32>,
    }
    struct I32 {
        res: i32,
    }
    struct OptionF64 {
        res: Option<f64>,
    }

    macro_rules! sql_func {
        ($type:ident, $func:ident, $res:expr) => {

    assert_eq!(select_as![$type, $func(id) AS res FROM Maison]
        .fetch_one(&p.pool)
        .await
        .unwrap().res, $res)
        };
    }
    sql_func!(OptionI32, SUM, Some(6));
    sql_func!(OptionI32, MIN, Some(1));
    sql_func!(OptionI32, MAX, Some(3));
    sql_func!(I32, COUNT, 3);
    sql_func!(OptionF64, AVG, Some(2f64));

    assert_eq!(
        select_as![OptionI32, SUM(a.id) AS res FROM Maison a]
            .fetch_one(&p.pool)
            .await
            .unwrap()
            .res,
        Some(6)
    );

    // #Literal
    #[derive(Debug, PartialEq)]
    struct Literal {
        string: String,
        // cchar: char,
        int: i32,
        float: f64,
        bbool: bool,
    }

    let res =
        select_as![Literal, "bla" AS string, 4 AS int, 5.6 AS float, true AS "bbool:_" FROM Maison]
            .fetch_one(&p.pool)
            .await
            .unwrap();
    assert_eq!(
        res,
        Literal {
            string: "bla".to_string(),
            int: 4,
            float: 5.6,
            bbool: true
        }
    );

    // Operation
    // simple
    let res = select_as![I32, id + 3 AS res FROM Maison ]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[1].res, 5);

    // with function
    let res = select_as![OptionI32, SUM(id)+COUNT(id) AS res FROM Maison ]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].res, Some(9));

    // all arythmetic operator
    let res = select_as![I32, id*id - id/id AS res FROM Maison ]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].res, 0);
    assert_eq!(res[1].res, 3);

    // all logic operator
    let res = select_as![I32, 0 || id==id && id!=taille AS res FROM Maison ]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].res, 1);
    assert_eq!(res[1].res, 1);

    // parenthesis
    let res = select_as![I32, (id+id)/id AS res FROM Maison]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].res, 2);
    assert_eq!(res[1].res, 2);

    // parenthes with function and field
    let res = select_as![OptionI32, (SUM(a.id)+COUNT(a.id)) + 2 AS res FROM Maison a]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].res, Some(11));
}
