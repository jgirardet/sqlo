use syn::{parse::ParseStream, Token};

pub mod kw {

    syn::custom_keyword!(order_by);
    syn::custom_keyword!(limit);
    syn::custom_keyword!(page);
    syn::custom_keyword!(group_by);
    syn::custom_keyword!(having);
}
pub fn next_is_not_a_keyword(input: &ParseStream) -> bool {
    !input.peek(Token![where])
        && !input.peek(kw::order_by)
        && !input.peek(kw::limit)
        && !input.peek(kw::page)
        && !input.peek(kw::group_by)
        && !input.peek(kw::having)
}
