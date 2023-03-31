use crate::macros::Generator;

#[derive(Debug, Clone, Copy)]
pub enum WhichMacro {
    Query,
    QueryAs,
}

impl WhichMacro {
    pub fn for_select(gen: &Generator) -> Self {
        if gen.query_parts.customs && gen.custom_struct.is_none() {
            Self::Query
        } else {
            Self::QueryAs
        }
    }
}
