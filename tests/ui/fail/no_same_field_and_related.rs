#[derive(sqlo::Sqlo)]
struct Bla {
    id: i32,
    un: String,
}

#[derive(sqlo::Sqlo)]
struct Bli {
    id: i32,
    #[sqlo(fk = "Bla", related = "un")]
    deux: i32,
}

fn main() {}
// the error message has many error, the important one is about related and field name
