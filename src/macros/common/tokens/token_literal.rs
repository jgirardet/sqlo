use crate::macros::common::{SelectContext, Sqlize, Sqlized};
use syn::{ExprLit, Lit};

use crate::macros::common::Validate;

#[derive(Debug)]
pub struct TokenLit {
    lit: Lit,
}

impl_trait_to_tokens_for_tokens!(TokenLit, lit);

impl TryFrom<ExprLit> for TokenLit {
    type Error = syn::Error;

    fn try_from(value: ExprLit) -> Result<Self, Self::Error> {
        Ok(TokenLit { lit: value.lit })
    }
}

impl Validate for TokenLit {}

impl Sqlize for TokenLit {
    fn sselect(&self, acc: &mut Sqlized, _context: &SelectContext) -> syn::Result<()> {
        let val = match &self.lit {
            Lit::Str(s) => {
                let res = format!("'{}'", s.value());
                res
            }
            Lit::Int(i) => i.base10_digits().to_string(),
            Lit::Float(f) => f.base10_digits().to_string(),
            Lit::Bool(b) => {
                if b.value {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            _ => return_error!(&self.lit, "Literal not supported"),
        };
        acc.append_sql(val);
        Ok(())
    }
}

#[cfg(test)]
impl crate::macros::common::stringify::Stringify for TokenLit {
    fn stry(&self) -> String {
        match &self.lit {
            Lit::Str(s) => format!("\"{}\"", s.value()),
            Lit::Bool(b) => b.value().to_string(),
            Lit::Int(i) => i.base10_digits().to_string(),
            Lit::Float(f) => f.base10_digits().to_string(),
            _ => unimplemented!("byte and bytestr not implemented"),
        }
    }
}
