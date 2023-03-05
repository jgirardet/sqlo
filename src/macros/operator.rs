use crate::macros::ColumnToSql;

#[derive(Debug)]
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
    Not,
}
macro_rules! impl_parse_for_operator {
    ($input:ident; $start:ident, $start_sign:tt, $($iden:ident,$sign:tt),+) => {
        if $input.peek(syn::Token![+]) {
            $input.parse::<syn::Token![+]>()?;
            return Ok(Operator::Add)
        }
        $(
        else if $input.peek(syn::Token![$sign]) {
            $input.parse::<syn::Token![$sign]>()?;
            return Ok(Operator::$iden)
        }
        )+
        else {
            return Err($input.error("Operator not supported"));
        }
    };
}
impl syn::parse::Parse for Operator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        impl_parse_for_operator!(input;
            Add,+, Sub,-, Mul,*, Div,/, Mod,%,
            Lt,<, Gt,>, Ge,>=, Le, <=,
            Eq,==, Neq, !=,
            And,&&, Or,||, Not, !
        )
    }
}

impl ColumnToSql for Operator {
    fn column_to_sql(
        &self,
        _ctx: &mut crate::macros::SqlResult,
    ) -> Result<crate::macros::SqlQuery, crate::error::SqloError> {
        match self {
            Self::Add => Ok("+".into()),
            Self::Sub => Ok("-".into()),
            Self::Mul => Ok("*".into()),
            Self::Div => Ok("/".into()),
            Self::Mod => Ok("%".into()),
            Self::Lt => Ok("<".into()),
            Self::Gt => Ok(">".into()),
            Self::Ge => Ok(">=".into()),
            Self::Le => Ok("<=".into()),
            Self::Eq => Ok("=".into()),
            Self::Neq => Ok("<>".into()),
            Self::And => Ok("AND".into()),
            Self::Or => Ok("OR".into()),
            Self::Not => Ok("NOT".into()),
        }
    }
}
