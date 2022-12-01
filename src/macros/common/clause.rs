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
            parse_select(&input)
        } else if input.peek(kw::WHERE) {
            parse_where(&input)
        } else if input.peek(kw::FROM) {
            parse_from(&input)
        } else {
            parse_select(&input)
        }
    }
}

fn parse_select(input: &syn::parse::ParseStream) -> syn::Result<Clause> {
    if input.peek(kw::SELECT) {
        input.parse::<kw::SELECT>()?;
    }
    Ok(Clause::Select(input.parse::<TokenSeq>()?.into()))
}

fn parse_from(input: syn::parse::ParseStream) -> syn::Result<Clause> {
    input.parse::<kw::FROM>()?;
    Ok(Clause::From(input.parse::<TokenSeq>()?.into()))
}

fn parse_where(input: &syn::parse::ParseStream) -> syn::Result<Clause> {
    input.parse::<kw::WHERE>()?;
    Ok(Clause::Where(input.parse::<TokenSeq>()?.into()))
}

#[cfg(test)]
mod test_clause {
    use super::*;

    #[test]
    fn select() {
        syn::parse_str::<Clause>("SELECT a,b, d,c").unwrap();
        syn::parse_str::<Clause>("SELECT a,b AS d,c").unwrap();
        syn::parse_str::<Clause>("SELECT a,b.d,c").unwrap();
        syn::parse_str::<Clause>("SELECT a,COUNT(b), \"zefze\", d,c").unwrap();
        syn::parse_str::<Clause>("a,COUNT(b), \"zefze\", d,c").unwrap();
        syn::parse_str::<Clause>("a AS b").unwrap();
        syn::parse_str::<Clause>("a(bla) AS b").unwrap();
        syn::parse_str::<Clause>("a.d AS b").unwrap();
    }

    #[test]
    fn wheres() {
        let res = syn::parse_str::<Clause>("WHERE count(bla)").unwrap();
        match res {
            Clause::Where(_) => {}
            _ => panic!("Not a Where Clause"),
        }
    }

    #[test]
    fn froms() {
        let res = syn::parse_str::<Clause>("FROM aaa, bbb b, ccc").unwrap();
        match res {
            Clause::From(_) => {}
            _ => panic!("Not a From Clause"),
        }
    }
}
