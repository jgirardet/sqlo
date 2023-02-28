use crate::{sqlo::Sqlo, sqlos::Sqlos};

pub struct Context<'a> {
    pub main_sqlo: &'a Sqlo,
    pub sqlos: &'a Sqlos,
}
