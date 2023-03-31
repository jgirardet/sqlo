use sqlo::Sqlo;
#[derive(sqlo::Sqlo)]
struct Adresse {
    id: std::string::String,
    rue: Option<String>,
}

#[derive(Sqlo)]
#[sqlo(tablename = "maison")]
struct Maison1 {
    id: i64,
    #[sqlo(fk = "Adresse")]
    adresse: std::string::String, //keep to test long path
    // #[sqlo(fk = "Adresse", fk_field = "id")]
    taille: i64,
    piscine: Option<bool>,
}

#[derive(Sqlo, PartialEq, Debug)]
#[sqlo(tablename = "piece")]
struct ExpandPiece {
    #[sqlo(primary_key, type_override, insert_fn = "uuid::Uuid::new_v4")]
    nb: uuid::Uuid,
    #[sqlo(fk = "Maison1")]
    la: i64,
    #[sqlo(fk = "Maison")]
    maison_id: i64,
}

fn main() {}
