use crate::PPool;
use sqlo::{sqlo, sqlo_as};

struct I64 {
    id: i64,
}

struct I32 {
    id: i32,
}

Test! {
    sqlo_macro_basic,
    async fn func(p: PPool) {
        let res = sqlo![SELECT id FROM Maison]
            .fetch_all(&p.pool)
            .await
            .unwrap();
        assert_eq!(res[0].id, 1);
        assert_eq!(res[1].id, 2);
        let res = sqlo![SELECT COUNT(id) AS bla FROM Maison]
            .fetch_one(&p.pool)
            .await
            .unwrap();
        assert_eq!(res.bla, 3);
    }
}

Test! {sqlo_macro_type_override_inline, async fn func(p: PPool) {
    let res = sqlo!(SELECT nb AS "nb: uuid::Uuid" FROM WithAttrs)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].nb, uu4!(1))
}}

Test! {sqlo_macro_type_override_inline_rename_field, async fn func(p: PPool) {
    let res = sqlo!(SELECT nb AS "noob: uuid::Uuid" FROM WithAttrs)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].noob, uu4!(1))
}}

Test! {
    sqlo_all_select_sqltoken_type_call_alone, async fn func(p:PPool) {
    let res2 = sqlx::query!("SELECT Max(a.id) AS c FROM maison a, piece b").fetch_one(&p.pool).await.unwrap();
    assert_eq!(res2.c, Some(3));
    }
}

#[rustfmt::skip]
Test! {sqlo_all_select_sqltoken_type, async fn func(p:PPool) {

    let res = sqlo!(SELECT

        id,
        "astring" AS astring,
        b.lglg,
        b.lglg AS thefield,
        1 +1 AS numeric_binary,
        1 && 1 AS "binary_bool:bool",
        id + id AS column_binary,
        id + 3 AS mixed_binary,
        (id) AS paren,
        (b.lglg) AS paren_field,
        COUNT(DISTINCT[id]) AS call_one_param,
        REPLACE(adresse, "adr", "ADR") AS call_multi_param
        FROM Maison, WithAttrs AS b).fetch_one(&p.pool).await.unwrap();

    assert_eq!(res.id, Some(1));
    assert_eq!(res.astring, "astring".to_string());
    assert_eq!(res.lglg, Some(1));
    assert_eq!(res.thefield, Some(1));
    assert_eq!(res.numeric_binary, 2 );
    assert_eq!(res.binary_bool, true );
    assert_eq!(res.column_binary, Some(2) );
    assert_eq!(res.mixed_binary, Some(4));
    assert_eq!(res.paren, Some(1));
    assert_eq!(res.paren_field, 1);
    assert_eq!(res.call_one_param, Some(3));
    assert_eq!(res.call_multi_param, Some("ADResse1".to_string()));
}}

// Test! {sqlo_where, async fn func(p:PPool) {
//     let res = sqlo![SELECT COUNT(nb) AS b FROM PieceFk WHERE maison_id == 1 ].fetch_one(&p.pool).await.unwrap();
//     assert_eq!(res.b, 4);
// }
// }

// ############################################################################################
// #                                                                                          #
// #                              SQLO_AS                                                     #
// #                                                                                          #
// ############################################################################################

Test![
    sqlo_as_macro,
    async fn func(p: PPool) {
        let res = sqlo_as![I64, SELECT id FROM Maison]
            .fetch_all(&p.pool)
            .await
            .unwrap();
        assert_eq!(res[0].id, 1);
        assert_eq!(res[1].id, 2);

        let res = sqlo_as![I32, SELECT COUNT(id) AS id FROM Maison]
            .fetch_one(&p.pool)
            .await
            .unwrap();
        assert_eq!(res.id, 3);
    }
];

Test! {sqlo_as_macro_type_override, async fn func(p: PPool) {
    struct Uuu {
        nb: uuid::Uuid,
    }
    let res = sqlo_as!(Uuu, SELECT nb FROM WithAttrs)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].nb, uu4!(1))
}}

Test! {sqlo_as_macro_type_override_inline, async fn func(p: PPool) {
    struct Uuu {
        aaaa: uuid::Uuid,
    }
    let res = sqlo_as!(Uuu, SELECT nb AS "aaaa:uuid::Uuid"  FROM WithAttrs)
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].aaaa, uu4!(1))
}}

#[rustfmt::skip]
Test! {sqlo_as_all_select_sqltoken_type, async fn func(p:PPool) {
    struct Res {
        id: i64,
        astring: String,
        la: i64,
        thefield: i64,
        numeric_binary: i32,
        binary_bool: bool,
        column_binary: i32,
        mixed_binary: i32,
        paren: i64,
        paren_field: i64,
        call_multi_param: Option<String>
        
    }

    let res = sqlo_as!(Res,SELECT
        id,
        "astring" AS astring,
        b.la,
        b.lglg AS thefield,
        1 +1 AS numeric_binary,
        1 && 1 AS "binary_bool:bool",
        id + id AS column_binary,
        id + 3 AS mixed_binary,
        (id) AS paren,
        (b.lglg) AS paren_field,
        // COUNT(DISTINCT:id) AS call_one_param // no sens inside this query
        REPLACE(adresse, "adr", "ADR") AS call_multi_param
        FROM Maison, WithAttrs b)
        .fetch_one(&p.pool).await.unwrap();

    assert_eq!(res.id, 1);
    assert_eq!(res.astring, "astring".to_string());
    assert_eq!(res.la, 10);
    assert_eq!(res.thefield, 1);
    assert_eq!(res.numeric_binary, 2 );
    assert_eq!(res.binary_bool, true );
    assert_eq!(res.column_binary, 2 );
    assert_eq!(res.mixed_binary, 4);
    assert_eq!(res.paren, 1);
    assert_eq!(res.paren_field, 1);
    assert_eq!(res.call_multi_param, Some("ADResse1".to_string()));
}}
