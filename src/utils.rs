macro_rules! parse_possible_bracketed {
    ($input:expr, $reste:ident) => {
        let content;
        $reste = if $input.peek(syn::token::Bracket) {
        syn::bracketed!(content in $input);
        &content
        } else {
        $input
         }
    };
}

pub const INSERT_FN_FLAG: &str = "ERGKE23YUKYUK4590C2";
