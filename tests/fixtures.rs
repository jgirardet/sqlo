#![allow(unused)]
// only run tu generate test fixtures

#[derive(sqlo::Sqlo)]
#[sqlo(parse_only)]
struct Aaa {
    id: i64,
    fstring: String,
    #[sqlo(column = "fi32col")]
    fi32: i32,
    foption: Option<String>,
}
#[derive(sqlo::Sqlo)]
#[sqlo(parse_only)]
struct Bbb {
    #[sqlo(primary_key)]
    uu: uuid::Uuid,
    #[sqlo(column = "fstringcol")]
    fstring: String,
    fi32: i32,
    foption: Option<String>,
    #[sqlo(fk = "Aaa")]
    aaa_fk: i64,
}

#[derive(sqlo::Sqlo)]
#[sqlo(parse_only, tablename = "ccctable")]
struct Ccc {
    id: i64,
    #[sqlo(fk = "Bbb")]
    bbb_fk: uuid::Uuid,
    height: i32,
}

#[derive(sqlo::Sqlo)]
#[sqlo(parse_only)]
struct Ddd {
    id: String,
    #[sqlo(fk = "Aaa", related = "the_ddds")]
    aaa_if: i64,
    size: i32,
}

#[test]
fn main() {}
