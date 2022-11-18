use super::{
    kw,
    sql::{is_a_keyword, ToSql},
    table::Tables,
};

#[derive(Debug, Default)]
pub struct Distinct(bool);

impl syn::parse::Parse for Distinct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::DISTINCT) {
            input.parse::<kw::DISTINCT>()?;
            Ok(Self(true))
        } else if is_a_keyword(input) {
            if input.peek(kw::distinct) {
                Err(syn::Error::new(input.span(), "Did you mean 'DISTINCT' ?"))
            } else {
                Err(syn::Error::new(
                    input.span(),
                    "Only keyword 'DISTINCT' expected here.",
                ))
            }
        } else {
            Ok(Self(false))
        }
    }
}

impl ToSql for Distinct {
    fn to_sql(&self, _: &Tables) -> syn::Result<String> {
        match self.0 {
            true => Ok("DISTINCT".to_string()),
            false => Ok("".to_string()),
        }
    }
}
