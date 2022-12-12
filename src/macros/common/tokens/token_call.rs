use syn::{Expr, ExprCall};

use crate::macros::common::{SelectContext, Sqlize, Sqlized, Validate};

use super::{token_seq::TokenSeq, TokenIdent};

#[derive(Debug)]
pub struct TokenCall {
    func: TokenIdent,
    content: TokenSeq,
}

impl_trait_to_tokens_for_tokens!(TokenCall, func, content);

impl TryFrom<Expr> for TokenCall {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        if let Expr::Call(ExprCall { func, args, .. }) = expr {
            return Ok(TokenCall {
                func: (*func).try_into()?,
                content: args.try_into()?,
            });
        }
        return_error!(expr, "invalid expression: not a call expression")
    }
}

#[rustfmt::skip]
const KNOWN_SQL_FUNCTIONS: &[&str] = &[
    // Aggregation
    "SUM", "COUNT", "MAX", "MIN", "AVG",
    // Strings
    "CONCAT", "LENGTH", "REPLACE", "SOUNDEX", "SUBSTRING", "LEFT", "RIGHT", "RIGHT", 
    "REVERSE", "TRIM", "LTRIM", "RTRIM", "LPAD", "UPPER", "LOWER", "UCASE",
    "LCASE", "LOCATE", "INSTR",
    // Math
    "RAND", "ROUND",
    // Datetime
    "DATE_FORMAT", "DATEDIFF", "DAYOFWEEK", "MONTH", "NOW", "SEC_OF_TIME", "TIMEDIFF",
    "TIMESTAMP", "YEAR",
    // MD5
    "MD5",
    // Conversion
    "CAST", "CONVERT", "GROUP_CONCAT", "IS_NULL", "VERSION"
];

impl Validate for TokenCall {
    fn validate(&self, _sqlos: &crate::sqlos::Sqlos) -> syn::Result<()> {
        let ident = self.func.to_string();
        if !KNOWN_SQL_FUNCTIONS.contains(&ident.as_str()) {
            if KNOWN_SQL_FUNCTIONS.contains(&ident.to_uppercase().as_str()) {
                return_error!(
                    &self.func,
                    &format!("Did you mean `{}` ?", &self.func.to_string().to_uppercase())
                )
            } else if ident.to_uppercase() != ident {
                return_error!(&self.func, "SQL functions must be uppercase.")
            } else {
                return Ok(());
            }
        }
        Ok(())
    }
}

impl Sqlize for TokenCall {
    fn sselect(&self, acc: &mut Sqlized, context: &SelectContext) -> syn::Result<()> {
        let mut group = Sqlized::default();
        group.append_sql(self.func.to_string());
        group.append_sql("(".to_string());
        self.content.sselect(&mut group, context)?;
        group.append_sql(")".to_string());
        acc.append_group(group);
        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenCall {
    fn stry(&self) -> String {
        format!("{}({})", &self.func.stry(), &self.content.stry())
    }
}

#[cfg(test)]
mod token_call {
    use crate::{macros::common::tokens::SqlToken, virtual_file::VirtualFile};

    use super::*;

    #[test]
    fn validate_method_name() {
        let sqlos = VirtualFile::new().load().unwrap();
        // success
        for func in KNOWN_SQL_FUNCTIONS {
            let t: SqlToken = syn::parse_str::<SqlToken>(&format!("{}(a)", func)).unwrap();
            assert!(t.validate(&sqlos).is_ok());
        }

        // bad function name
        let t: SqlToken = syn::parse_str::<SqlToken>("bla(a)").unwrap();
        assert_eq!(
            t.validate(&sqlos).err().unwrap().to_string(),
            "SQL functions must be uppercase.",
        );

        // upper/lowercase
        let t: SqlToken = syn::parse_str::<SqlToken>("count(a)").unwrap();
        assert_eq!(
            t.validate(&sqlos).err().unwrap().to_string(),
            "Did you mean `COUNT` ?",
        );
    }
}
