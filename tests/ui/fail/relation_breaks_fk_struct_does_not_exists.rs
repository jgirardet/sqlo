use sqlo::Sqlo;
#[derive(Sqlo)]
#[sqlo(tablename = "maison")]
struct Maison1 {
    id: i64,
    // // #[sqlo(fk = "Adresse", fk_field = "id")]
    taille: i64,
    // #[sqlo(fk = "Piece1")]
    piscine: Option<bool>,
}

#[derive(Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "piece")]
struct ExpandPiece {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    nb: uuid::Uuid,
    #[sqlo(fk = "SomeWrongStruct")]
    maison_id: i64,
}

fn main(){}