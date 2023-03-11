// without any attr
#[derive(sqlo::Sqlo, Debug, PartialEq, Eq)]
pub struct Maison {
    pub id: i64,
    pub adresse: String,
    pub taille: i64,
    pub piscine: Option<bool>,
}

// with a single attr in sqlo attr
#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
#[sqlo(tablename = "piece")]
pub struct WithAttrs {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    pub nb: uuid::Uuid, // keep full path
    #[sqlo(type_override, column = "lg")]
    pub lglg: i32,
    pub la: i64,
    pub maison_id: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
#[sqlo(tablename = "maison")]
pub struct Maison2 {
    #[sqlo(create_arg, type_override)]
    pub id: i32,
    pub adresse: String,
    pub taille: i64,
    pub piscine: Option<bool>,
}

#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
pub struct Adresse {
    #[sqlo(create_arg)]
    pub id: String,
    pub rue: Option<String>,
    #[sqlo(fk = "Maison", related = "adres")]
    pub m_id: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
#[sqlo(tablename = "piece")]
pub struct PieceFk {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    pub nb: uuid::Uuid, // keep full path
    #[sqlo(type_override)]
    pub lg: i32,
    pub la: i64,
    #[sqlo(fk = "Maison", related = "lespieces")]
    pub maison_id: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
#[sqlo(tablename = "piece")]
pub struct PieceFk2 {
    #[sqlo(primary_key, type_override, create_fn = "uuid::Uuid::new_v4")]
    pub nb: uuid::Uuid, // keep full path
    #[sqlo(type_override)]
    pub lg: i32,
    pub la: i64,
    #[sqlo(fk = "Maison")]
    pub maison_id: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
pub struct Lit {
    pub id: i64,
    pub surface: i64,
}

#[derive(sqlo::Sqlo, PartialEq, Eq, Debug)]
pub struct SelfRelation {
    id: i64,
    name: String,
    salary: i64,
    #[sqlo(fk = "SelfRelation", related = "manager")]
    manager_id: Option<i64>,
}
