use syn::{ExprLit, Lit};

#[derive(Debug)]
pub struct TokenLit {
    lit: Lit,
}

impl_to_tokens_for_tokens!(TokenLit, lit);

impl TryFrom<ExprLit> for TokenLit {
    type Error = syn::Error;

    fn try_from(value: ExprLit) -> Result<Self, Self::Error> {
        Ok(TokenLit { lit: value.lit })
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
