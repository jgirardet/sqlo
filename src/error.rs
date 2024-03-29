use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

#[derive(Debug)]
pub struct SqloError {
    msg: String,
    span: Span,
}

impl SqloError {
    pub fn new(msg: &str, span: Span) -> Self {
        SqloError {
            msg: msg.to_string(),
            span,
        }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn new_spanned<T: ToTokens, U: Display>(tokens: T, message: U) -> Self {
        syn::Error::new_spanned(tokens, message).into()
    }

    pub fn new_lost(msg: &str) -> Self {
        SqloError {
            msg: msg.to_string(),
            span: Span::call_site(),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_compile_error(self) -> TokenStream {
        syn::Error::from(self).to_compile_error()
    }
}

pub trait ToSqloError<T> {
    fn sqlo_err(self, span: Span) -> Result<T, SqloError>;
}

impl<T, E: std::error::Error> ToSqloError<T> for Result<T, E> {
    /// Convert Result<T,E> to Result<T,SqloError> with Span.
    fn sqlo_err(self, span: Span) -> Result<T, SqloError> {
        match self {
            Ok(m) => Ok(m),
            Err(e) => Err(SqloError::new(&e.to_string(), span)),
        }
    }
}

impl Display for SqloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for SqloError {}

impl From<std::io::Error> for SqloError {
    fn from(e: std::io::Error) -> Self {
        SqloError {
            msg: e.to_string(),
            span: Span::call_site(),
        }
    }
}

impl From<std::fmt::Error> for SqloError {
    fn from(e: std::fmt::Error) -> Self {
        SqloError {
            msg: e.to_string(),
            span: Span::call_site(),
        }
    }
}

impl From<SqloError> for std::io::Error {
    fn from(e: SqloError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

impl From<SqloError> for syn::Error {
    fn from(s: SqloError) -> Self {
        syn::Error::new(s.span, &s.msg)
    }
}

impl serde::ser::Error for SqloError {
    fn custom<T: Display>(msg: T) -> Self {
        SqloError {
            msg: msg.to_string(),
            span: Span::call_site(),
        }
    }
}

impl serde::de::Error for SqloError {
    fn custom<T: Display>(msg: T) -> Self {
        SqloError {
            msg: msg.to_string(),
            span: Span::call_site(),
        }
    }
}

impl From<serde_json::Error> for SqloError {
    fn from(e: serde_json::Error) -> Self {
        Self::new_lost(&e.to_string())
    }
}

impl From<syn::Error> for SqloError {
    fn from(e: syn::Error) -> Self {
        Self::new(&e.to_string(), e.span())
    }
}
