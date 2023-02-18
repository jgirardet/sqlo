use std::fmt::Display;

use darling::util::IdentString;

use crate::{macros::common::TokenIdent, sqlo::Sqlo, sqlos::Sqlos};

use super::to_alias_sqlos::ToAliasSqlos;

#[derive(Debug, Clone)]
pub enum AliasSqlo<'a, 'b> {
    Ident(&'a Sqlo),
    Alias(&'a Sqlo, &'b TokenIdent),
}

impl<'a, 'b> AliasSqlo<'a, 'b> {
    pub fn ident(&self) -> &IdentString {
        match self {
            Self::Alias(_, a) => a.as_ref(),
            Self::Ident(s) => &s.ident,
        }
    }

    pub fn sqlo(&self) -> &Sqlo {
        match self {
            Self::Alias(s, _) => s,
            Self::Ident(s) => s,
        }
    }
}

type VecSqlo<'a, 'b> = Vec<AliasSqlo<'a, 'b>>;

impl<'a, 'b> Display for AliasSqlo<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
        //     AliasSqlo::Alias(alias) => write!(f, "{}", alias.alias.unwrap()), //unwrap is ok since checked at built
        //     FoundAlias::Ident(alias) => write!(f, "{}", alias.sqlo.ident),
        // }
        write!(f, "{}", self.ident())
    }
}

#[derive(Debug)]
// pub struct AliasSqlos<'a, 'b>(VecSqlo<'a, 'b>);

pub struct AliasSqlos<'a, 'b>(VecSqlo<'a, 'b>);

impl<'a, 'b> AliasSqlos<'a, 'b> {
    pub fn find(&self, name: &TokenIdent) -> Result<&AliasSqlo, syn::Error> {
        for al in &self.0 {
            match al {
                AliasSqlo::Alias(s, a) => {
                    if &name == a {
                        return Ok(al);
                    }
                    // else {
                    //     continue; // alias exists but don't match, so don't use tablename like in SQL
                    // }
                }
                AliasSqlo::Ident(s) => {
                    if &s.ident == name {
                        return Ok(&al);
                    }
                }
            }
        }
        return_error!(name, &format!("Can't find Sqlo struct {}", name))
    }

    pub fn new() -> Self {
        AliasSqlos(vec![])
    }

    pub fn iter(&self) -> std::slice::Iter<'_, AliasSqlo<'a, 'b>> {
        self.0.iter()
    }

    pub fn extend<T>(&mut self, clause: T, sqlos: &'b Sqlos) -> syn::Result<()>
    where
        T: ToAliasSqlos<'b, 'a> + 'a,
    {
        let other_alias = clause.to_alias_sqlos(sqlos)?;
        // for alias in other_alias.iter() {
        //     // self.0.push(*alias)
        // }
        Ok(())
    }
}

impl<'a, 'b> From<VecSqlo<'a, 'b>> for AliasSqlos<'a, 'b> {
    fn from(v: VecSqlo<'a, 'b>) -> Self {
        AliasSqlos(v)
    }
}
