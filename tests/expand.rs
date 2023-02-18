#![allow(dead_code)]
#![allow(unused_variables)]

use sqlo::Sqlo;
use syn::Token;

// #[derive(Sqlo)]
// struct Adresse {
//     id: String,
//     rue: Option<String>,
// }
#[derive(Sqlo, Debug, PartialEq)]
#[sqlo(tablename = "maison")]
struct ExpandMaison {
    #[sqlo(type_override, create_fn = "uuid::Uuid::new_v4")]
    id: i64,
    // #[sqlo(fk = "Adresse")]
    #[sqlo(column = "adresse")]
    adr: String,
    // #[sqlo(fk = "Adresse", fk_field = "id")]
    taille: i64,
    // #[sqlo(fk = "ExpandPiece")]
    piscine: Option<bool>,
}

#[derive(Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "piece")]
struct ExpandPiece {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    nb: uuid::Uuid, // keep full path
    #[sqlo(type_override, column = "lg")]
    lglg: i32,
    // #[sqlo(fk = "ExpandMaison", fk_field = "lefield")]
    la: i64,
    #[sqlo(fk = "ExpandMaison", related = "pieces")]
    maison_id: i64,
}

// #[derive(Debug, Sqlo)]
// struct IdUniqueUuid {
//     #[sqlo(create_fn = "uuid::Uuid::new_v4")]
//     id: uuid::Uuid,

// }
//

#[derive(Debug)]
struct In {
    left: Box<syn::Expr>,
    right: Box<syn::Expr>,
}

impl syn::parse::Parse for In {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left = input.parse()?;
        input.parse::<Token![in]>()?;
        let right = input.parse()?;

        Ok(Self {
            left: Box::new(left),
            right: Box::new(right),
        })
    }
}

#[async_std::main]
async fn main() {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    // let res = sqlo_select!(ExpandMaison[1].pieces)
    //     .fetch_all(&pool)
    //     .await
    //     .unwrap();
    // // dbg!(&res);
    // let res = sqlx::query!(r#"SELECT 'piece'.la as "la:i32"  FROM piece"#)
    //     .fetch_one(&pool)
    //     .await
    //     .unwrap();
    #[derive(Debug)]
    struct Res {
        bla: String,
        taille: i64,
    }
    // let res = sqlx::query!["select 'hello' as x from maison"]
    // // let res = select_as![Res, taille, a.adr AS bla  FROM ExpandMaison a, ExpandPiece b ]
    // //     .fetch_all(&pool)
    // //     .await
    // //     .unwrap();
    // dbg!(&res);
    // let res = sqlo::sqlo![SELECT "string" AS bla,1,true,false,1.2 FROM ExpandMaison]
    //     .fetch_one(&pool)
    //     .await
    //     .unwrap();
    // dbg!(&res);

    // sqlo::sqlo![SELECT a,CONCAT(b+c) AS bl, c.d FROM  aaa azd, bbb b  ];
    // use syn::parse::Parse;
    // let parsed: ExprGroup = syn::parse_str("a,b,c").unwrap();
    // dbg!(&parsed)
}

#[test]
fn test_main_expand() {
    main();
}
