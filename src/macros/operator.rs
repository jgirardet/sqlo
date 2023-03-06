use crate::macros::ColumnToSql;

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Ge,
    Le,
    Eq,
    Neq,
    And,
    Or,
    Like,
    In,
}
macro_rules! impl_parse_for_operator {
    ($start:ident, $start_sign:tt, $($iden:ident,$sign:tt),+) => {
    impl syn::parse::Parse for Operator {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            if input.peek(syn::Token![$start_sign]) {
                input.parse::<syn::Token![$start_sign]>()?;
                return Ok(Operator::$start)
            }
            $(
            else if input.peek(syn::Token![$sign]) {
                input.parse::<syn::Token![$sign]>()?;
                return Ok(Operator::$iden)
            }
            )+
            else {
                return Err(input.error("Operator not supported"));
            }
        }
    }
    }
}

macro_rules! impl_columnto_sql_for_operator {
    ($($nom:ident $op:literal),+) => {

    impl ColumnToSql for Operator {
        fn column_to_sql(
            &self,
            _ctx: &mut crate::macros::SqlResult,
        ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
            match self {
                $(Self::$nom => Ok($op.into())),+
            }
        }
    }
    };
}

macro_rules! impl_to_tokens_for_operator {
    ($($nom:ident $op:literal),+) => {
    impl quote::ToTokens for Operator {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                $(Self::$nom => $op.to_tokens(tokens)),+
            };
        }
    }
    };
}

macro_rules! impl_is_next_supported_operator {
    ($start:tt, $($other:tt),+) => {
    impl Operator {
    pub fn next_is_supported_op(input: &syn::parse::ParseStream) -> bool {
        input.peek(syn::Token![$start])
            $(|| input.peek(syn::Token![$other]))+
    }
    }
    };
}

impl_parse_for_operator!(
    Add,+, Sub,-, Mul,*, Div,/, Mod,%,
    Eq,==, Neq, !=, Ge,>=, Le, <=,Lt,<, Gt,>, //order matter composed before single token
    And,&&, Or,||, //Not, !
    Like,#, In, in
);

impl_columnto_sql_for_operator!(
    Add "+", Sub "-", Mul "*", Div "/", Mod "%",
    Ge ">=", Le "<=", Lt "<", Gt ">",  Eq "=", Neq "<>",
    And "AND", Or "OR",
    Like "LIKE", In "IN"

);

impl_to_tokens_for_operator!(
    Add "+", Sub "-", Mul "*", Div "/", Mod "%",
    Ge ">=", Le "<=", Lt "<", Gt ">", Eq "==", Neq "!=",
    And "&&", Or "||",
    Like "#", In "in"
);
// Self::Not => Ok("NOT".into()),

impl_is_next_supported_operator!(
    +, -, *, /, %,
    <=, >=, <, >, ==, !=,
    &&, ||,
    #, in
);
