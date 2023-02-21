use darling::util::IdentString;

pub enum Column {
    Ident(IdentString),
    Cast(ColumnCast),
}

impl syn::parse::Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr_base = input.parse::<syn::Expr>()?;
        match expr_base {
            syn::Expr::Cast(syn::ExprCast { expr, ty, .. }) => match ty.as_ref() {
                syn::Type::Path(syn::TypePath { path, .. }) => {
                    if let Some(ident) = path.get_ident() {
                        let alias = IdentString::new(ident.clone());
                        return Ok(Column::Cast(ColumnCast { expr: *expr, alias }));
                    }
                }
                _ => {}
            },
            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    return Ok(Column::Ident(ident.clone().into()));
                }
            }
            _ => return Err(input.error("column's expression should be followed by as")),
        }
        Err(input.error("custom column please use the following: ident or some(expr) as ident"))
    }
}

pub struct ColumnCast {
    pub expr: syn::Expr,
    pub alias: IdentString,
}
