use sqlo::Sqlo;

#[derive(Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "piece")]
struct ExpandPiece {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    nb: uuid::Uuid,
    maison_id: i64,
}

#[derive(Sqlo)]
#[sqlo(tablename = "maison")]
struct Maison1 {
    id: i64,
    #[sqlo(fk = "ExpandPiece")]
    taille: i64,
    piscine: Option<bool>,
}

fn main() {}
