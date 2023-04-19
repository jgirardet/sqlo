#![deny(unused_macros)]
use sqlo::Sqlo;

#[derive(Debug, PartialEq, Sqlo)]
struct Maison {
    id: i64,
    adresse: String, //do not remove (only one other field needed)
}
#[async_std::main]
async fn main() {
    let _a = "omjk";
}
//fail if unuwsed_macro warning appears
