use crate::macros::{InsertParser, SelectParser, UpdateParser};

use super::{QueryParser, TableAliases};
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Select,
    Update,
    Insert,
}

impl Mode {
    /// run the macros depending mode
    pub fn process(
        self,
        input: proc_macro::TokenStream,
    ) -> Result<proc_macro2::TokenStream, crate::SqloError> {
        match self {
            Self::Select => self.expand(syn::parse::<SelectParser>(input)?),
            Self::Update => self.expand(syn::parse::<UpdateParser>(input)?),
            Self::Insert => self.expand(syn::parse::<InsertParser>(input)?),
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
        let tables: TableAliases = TableAliases::new(&sqlos);
        let generator =
            crate::macros::Generator::from_sqlo_query_parse(self, parsed, &sqlos, false, tables)?;

        generator.expand(
            #[cfg(debug_assertions)]
            debug,
        )
    }
}
