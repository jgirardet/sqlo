use super::{keyword::kw, tokens::SqlToken, tokens::TokenSeq};

#[derive(Debug)]
pub enum Clause {
    Select(SqlToken),
    From(SqlToken),
    Where(SqlToken),
}

impl syn::parse::Parse for Clause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::SELECT) {
            parse_select(input)
        } else if input.peek(kw::WHERE) {
            parse_where(input)
        } else if input.peek(kw::FROM) {
            parse_from(input)
        } else {
            parse_select(input)
        }
    }
}

fn parse_select(input: syn::parse::ParseStream) -> syn::Result<Clause> {
    if input.peek(kw::SELECT) {
        input.parse::<kw::SELECT>()?;
    }
    Ok(Clause::Select(input.parse::<TokenSeq>()?.into()))
}

fn parse_from(input: syn::parse::ParseStream) -> syn::Result<Clause> {
    input.parse::<kw::FROM>()?;
    Ok(Clause::From(input.parse::<TokenSeq>()?.into()))
}

fn parse_where(input: syn::parse::ParseStream) -> syn::Result<Clause> {
    input.parse::<kw::WHERE>()?;
    Ok(Clause::Where(input.parse::<TokenSeq>()?.into()))
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for Clause {
    fn stry(&self) -> String {
        match self {
            Self::From(x) => format!("FROM {}", x.stry()),
            Self::Where(x) => format!("WHERE {}", x.stry()),
            Self::Select(x) => format!("SELECT {}", x.stry()),
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
        stry_cmp!("SELECT a,COUNT(b),\"zefze\",d,c", Clause);
        stry_cmp!(
            "a,COUNT(b),\"zefze\",d,c",
            Clause,
            "SELECT a,COUNT(b),\"zefze\",d,c"
        );
        stry_cmp!("SELECT a AS b", Clause);
        stry_cmp!("SELECT a(bla) AS b", Clause);
        stry_cmp!("SELECT a.d AS b", Clause);
        stry_cmp!("SELECT (a + 2) AS b", Clause);
        stry_cmp!("SELECT a(b,c)", Clause);

        stry_cmp!("WHERE count(bla)", Clause);

        stry_cmp!("FROM aaa,bbb b,ccc", Clause);
    }
}
