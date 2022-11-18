use super::table::Tables;

pub const SQL_FUNCTIONS: &[&str] = &["COUNT", "AVG", "MAX", "MIN", "SUM"];

pub trait ToSql {
    fn to_sql(&self, tables: &Tables) -> syn::Result<String>;
}

macro_rules! peek_kw {
    (k;$input:ident; $($k:ident),+) => {
       $($input.peek(syn::Token![$k]))||+
    };
    ($input:ident; $($k:ident),+) => {
       $($input.peek(crate::macros::select::kw::$k))||+
    };
}

pub fn is_a_keyword(input: &syn::parse::ParseBuffer) -> bool {
    peek_kw!(input; FROM, from, WHERE, JOIN, join, distinct, AS) || peek_kw!(k;input ; where, as)
}
