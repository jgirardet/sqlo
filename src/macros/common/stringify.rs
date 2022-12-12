#![cfg(test)]
/// Stringify token, tests purpose only

pub trait Stringify {
    fn stry(&self) -> String;

    fn assert_eq(&self, res: &str) {
        assert_eq!(self.stry().as_str(), res);
    }
}

macro_rules! stry_cmp {
    ($input:literal, $how:ty) => {
        let parsed: $how = syn::parse_str($input).unwrap();
        assert_eq!(
            $input,
            <$how as crate::macros::common::stringify::Stringify>::stry(&parsed)
        )
    };
    ($input:literal, $how:ty, $output:literal) => {
        let parsed: $how = syn::parse_str($input).unwrap();
        assert_eq!(
            $output,
            <$how as crate::macros::common::stringify::Stringify>::stry(&parsed)
        )
    };
}
