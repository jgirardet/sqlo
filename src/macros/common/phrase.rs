use std::slice;

use crate::macros::common::keyword::peek_keyword;

use super::clause::Clause;

pub struct Phrase(Vec<Clause>);

impl<'a> IntoIterator for &'a Phrase {
    type Item = &'a Clause;
    type IntoIter = slice::Iter<'a, Clause>;

    fn into_iter(self) -> slice::Iter<'a, Clause> {
        self.0.iter()
    }
}

impl syn::parse::Parse for Phrase {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut res = vec![];
        res.push(input.parse()?);
        while !input.is_empty() {
            if !peek_keyword(input) {
                return Err(input.error("Each clause should start with a keyword"));
            }
            res.push(input.parse()?)
        }
        Ok(Self(res))
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for Phrase {
    fn stry(&self) -> String {
        use itertools::Itertools;
        self.into_iter().map(|x| x.stry()).join(" ")
    }
}

#[cfg(test)]
mod test_phrase {
    use super::*;

    #[test]
    fn select_from_where() {
        stry_cmp!(
            "SELECT a,COUNT(b + c) AS bla,c.d FROM aaa,ccc c,bbb WHERE (a + 1) > 4 && c.d < COUNT(a)",
            Phrase
        );
    }
}
