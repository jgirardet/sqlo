use std::slice;

use proc_macro2::TokenStream;

use crate::sqlos::Sqlos;

use super::{
    clause::Clause, sqlize::Sqlized, AliasSqlos, FromContext, QueryContext, QueryMoment,
    SelectContext, Sqlize, ToAliasSqlos, Validate,
};

pub struct Phrase {
    clauses: Vec<Clause>,
}

impl<'a> IntoIterator for &'a Phrase {
    type Item = &'a Clause;
    type IntoIter = slice::Iter<'a, Clause>;

    fn into_iter(self) -> slice::Iter<'a, Clause> {
        self.clauses.iter()
    }
}

impl syn::parse::Parse for Phrase {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut clauses = vec![];
        while !input.is_empty() {
            clauses.push(input.parse()?)
        }
        Ok(Self { clauses })
    }
}

impl Phrase {
    pub fn sqlize(&self, sqlos: &Sqlos, query_context: QueryContext) -> syn::Result<Sqlized> {
        let mut acc = Sqlized::default();
        let mut iter = self.into_iter();

        let select = iter
            .next()
            .expect("Error query should start with select, update, ....");
        let alias_sqlos = AliasSqlos::new();

        while let Some(clause) = iter.next() {
            match clause {
                Clause::From(from) => {
                    let aliases = from.to_alias_sqlos(sqlos)?;
                    // let mut context_select =
                    //     SelectContext::from_clausefrom(&alias_sqlos, query_context)?;
                }
            }
        }

        if let Some(Clause::Select(ref sel)) = iter.next() {
            if let Some(Clause::From(from_clause)) = iter.next() {
                let alias_sqlos = from_clause.to_alias_sqlos(sqlos)?;
                let mut context_select =
                    SelectContext::from_clausefrom(&alias_sqlos, query_context)?;
                sel.sselect(&mut acc, &mut context_select)?;
                let context_from = FromContext::from_clausefrom(&alias_sqlos)?;
                from_clause.ffrom(&mut acc, &context_from)?;
            } else {
                return_error!(&sel.tokens, "a FROM clause should follow a SELECT clause")
            }
        }
        Ok(acc)
    }

    pub fn expand(self, sqlos: &Sqlos) -> syn::Result<TokenStream> {
        self.validate(sqlos)?;
        let sqlized = self.sqlize(sqlos, QueryContext::Sqlo(QueryMoment::InPhrase))?;
        let sql = sqlized.to_string();
        let params = sqlized.params();
        if std::env::var("SQLO_DEBUG_QUERY").is_ok() {
            dbg!(&sql);
        }
        Ok(quote::quote! {
            sqlx::query![#sql,#(#params),*]
        })
    }
}

impl Validate for Phrase {
    fn validate(&self, sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
        // first validate Clause
        for i in self {
            i.validate(sqlos)?
        }
        // then validate cross-Clause
        // is it needed  since we are passing context to sqlize methods?
        Ok(())
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
    use crate::virtual_file::VirtualFile;

    use super::*;

    #[test]
    fn select_from_where() {
        stry_cmp!(
        "SELECT a,COUNT(b + c) AS bla,c.d FROM aaa,ccc c,bbb WHERE (a + 1) > 4 && c.d < COUNT(a) AS b",
        Phrase
    );
    }

    #[test]
    fn validate_cascade() {
        let sqlos = VirtualFile::new().load().unwrap();
        let p: Phrase = syn::parse_str("SELECT bla(a) AS b FROM bli").unwrap();
        assert!(p.validate(&sqlos).is_err());
        if let Err(e) = p.validate(&sqlos) {
            assert_eq!(e.to_string(), "SQL functions must be uppercase.");
            return;
        }
        panic!("sould have failed")
    }

    #[test]
    fn distinct_in_select() {
        syn::parse_str::<Phrase>("SELECT DISTINCT a FROM bli").unwrap();
    }
}
