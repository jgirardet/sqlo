#[derive(sqlo::Sqlo)]
enum NoEnumStruct {}

#[derive(sqlo::Sqlo)]
struct NoTuppleStruct(String);

#[derive(sqlo::Sqlo)]
struct NoUnitStruct(String);

fn main() {}
