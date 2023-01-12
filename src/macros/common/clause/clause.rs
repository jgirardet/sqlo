use crate::macros::common::{kw, ClauseFrom, ClauseSelect, ClauseWhere, Validate};

pub enum Clause {
    Where(ClauseWhere),
    Select(ClauseSelect),
    From(ClauseFrom),
}

impl syn::parse::Parse for Clause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let res = if input.peek(kw::SELECT) {
            input.parse::<ClauseSelect>()?.into()
        } else if input.peek(kw::FROM) {
            input.parse::<ClauseFrom>()?.into()
        } else if input.peek(kw::WHERE) {
            input.parse::<ClauseWhere>()?.into()
        } else {
            input.parse::<ClauseSelect>()?.into()
        };
        Ok(res)
    }
}

impl Validate for Clause {
    fn validate(&self, sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
        match self {
            Self::From(x) => x.validate(sqlos),
            Self::Where(x) => x.validate(sqlos),
            Self::Select(x) => x.validate(sqlos),
        }
    }
}

impl quote::ToTokens for Clause {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::From(x) => x.to_tokens(tokens),
            Self::Where(x) => x.to_tokens(tokens),
            Self::Select(x) => x.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for Clause {
    fn stry(&self) -> String {
        match self {
            Self::From(x) => x.stry(),
            Self::Where(x) => x.stry(),
            Self::Select(x) => x.stry(),
        }
    }
}

#[cfg(test)]
mod test_clause {
    use super::*;

    #[test]
    fn clause() {
        stry_cmp!("SELECT a,b,d,c", Clause);
        stry_cmp!("SELECT a,b,d,c", Clause);
        stry_cmp!("SELECT a,b AS d,c", Clause);
        stry_cmp!("SELECT a,b.d,c", Clause);
        stry_cmp!("SELECT a,COUNT(b) AS b,\"zefze\" AS e,d,c", Clause);
        stry_cmp!(
            "a,COUNT(b) AS e,\"zefze\" AS f,d,c",
            Clause,
            "SELECT a,COUNT(b) AS e,\"zefze\" AS f,d,c"
        );
        stry_cmp!("SELECT a AS b", Clause);
        stry_cmp!("SELECT a(bla) AS b", Clause);
        stry_cmp!("SELECT a.d AS b", Clause);
        stry_cmp!("SELECT (a + 2) AS b", Clause);
        stry_cmp!("SELECT a(b,c) AS c", Clause);

        stry_cmp!("WHERE count(bla)", Clause);

        stry_cmp!("FROM aaa,bbb b,ccc", Clause);
    }
}
