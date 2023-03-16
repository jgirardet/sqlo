use crate::macros::SelectParser;

use super::QueryParser;
#[derive(Debug)]
pub enum Mode {
    Select,
    Update,
}

impl Mode {
    /// run the macros depending mode
    pub fn process(
        self,
        input: proc_macro::TokenStream,
    ) -> Result<proc_macro2::TokenStream, crate::SqloError> {
        match self {
            Self::Select => self.expand(syn::parse::<SelectParser>(input)?),
            _ => unimplemented!(),
        }
    }
}
impl Mode {
    pub fn expand<T>(self, parsed: T) -> Result<proc_macro2::TokenStream, crate::SqloError>
    where
        T: QueryParser,
    {
        #[cfg(debug_assertions)]
        let debug = parsed.debug();

        let sqlos = crate::VirtualFile::new().load()?;
        let generator = crate::macros::Generator::from_sqlo_query_parse(
            self,
            parsed,
            &sqlos,
            false,
            crate::macros::TableAliases::default(),
        )?;

        #[cfg(debug_assertions)]
        if debug {
            generator.debug();
        }

        generator.expand()
    }
}
