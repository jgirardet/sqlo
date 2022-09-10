use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;

pub struct SqloSetParse {
    tablename: syn::LitStr,
    pkfield: syn::Ident,
    instance: syn::Ident,
    values: Vec<syn::Expr>,
    pool: syn::Expr,
    columns: Vec<String>,
}

// sqlo_set{ "tablename" pk, &pool, inst, "fieldcolumn",  arg=value,arg=value}
impl syn::parse::Parse for SqloSetParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tablename = input.parse::<syn::LitStr>()?;
        let pkfield = input.parse::<syn::Ident>()?;
        input.parse::<Token!(,)>()?;
        let pool = input.parse::<syn::Expr>()?;
        input.parse::<Token!(,)>()?;
        let instance: syn::Ident = input.parse()?;
        input.parse::<Token!(,)>()?;
        let fieldcol = input.parse::<syn::LitStr>()?;
        input.parse::<Token!(,)>()?;
        let args = syn::punctuated::Punctuated::<syn::Expr, Token!(,)>::parse_terminated(input)?;
        // if args.is_empty() {
        //     panic!("datbaase empty");
        // }
        let mut fields: Vec<syn::Ident> = vec![];
        let mut values = vec![];
        for exp in args.into_iter() {
            if let syn::Expr::Assign(exp) = exp {
                let syn::ExprAssign { left, right, .. } = exp;
                if let syn::Expr::Type(syn::ExprType { expr, .. }) = *left {
                    if let syn::Expr::Path(syn::ExprPath { path, .. }) = *expr {
                        if let Some(ident) = path.get_ident() {
                            fields.push(ident.clone());
                            values.push(*right);
                        }
                    }
                }
            }
        }

        let field_col_value = fieldcol.value();
        let field_col = field_col_value
            .split(',')
            .map(|s| {
                let mut sp = s.split(':');
                (
                    sp.next()
                        .expect("sqlo_set error retrieving field/column")
                        .to_string(),
                    sp.next()
                        .expect("sqlo_set error retrieving field/column")
                        .to_string(),
                )
            })
            .collect::<Vec<_>>();

        let fields = fields.iter().map(|f| f.to_string()).collect::<Vec<_>>();
        let columns = field_col
            .into_iter()
            .filter_map(|(f, c)| {
                if fields.contains(&f.to_string()) {
                    Some(c)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(Self {
            tablename,
            instance,
            columns,
            values,
            pool,
            pkfield,
        })
    }
}

impl SqloSetParse {
    pub fn expand(&self) -> TokenStream {
        let instance = &self.instance;
        let columns_qmarks = self
            .columns
            .iter()
            .map(|c| format!("{c}=?"))
            .into_iter()
            .join(",");
        let tablename = &self.tablename;
        let pool = &self.pool;
        let pkfield = &self.pkfield;
        let values =
            syn::punctuated::Punctuated::<&syn::Expr, Token!(,)>::from_iter(self.values.iter());

        quote! {


            sqlx::query!("UPDATE " + #tablename  + " SET " + #columns_qmarks + " WHERE id=?;", #values, #instance.#pkfield).execute(#pool).await

        }
    }
}

pub fn process_sqlo_set(input: SqloSetParse) -> syn::Result<TokenStream> {
    dbg!(&input.columns);
    if input.columns.is_empty() {
        return Ok(quote! {});
    }
    Ok(input.expand())
}
