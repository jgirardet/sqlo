use crate::PPool;
use sqlo::{sqlo, sqlo_as};

struct I64 {
    un: i64,
}

struct I32 {
    res: i32,
}

pub async fn sqlo_macro(p: PPool) {
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

pub async fn sqlo_as_macro(p: PPool) {
    let res = sqlo_as![I64, SELECT id AS un FROM Maison]
        .fetch_all(&p.pool)
        .await
        .unwrap();
    assert_eq!(res[0].un, 1);
    assert_eq!(res[1].un, 2);

    let res = sqlo_as![I32, SELECT COUNT(id) AS res FROM Maison]
        .fetch_one(&p.pool)
        .await
        .unwrap();
    assert_eq!(res.res, 3);
}
