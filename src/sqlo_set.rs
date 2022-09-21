use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Token;

use crate::{
    query_builder::qmarks_with_col,
    sqlo::{DatabaseType, Sqlo},
};

pub struct SqloSetParse {
    sqlo: Sqlo,
    instance: syn::Ident,
    parse_values: Vec<syn::Expr>,
    pool: syn::Expr,
    parse_fields: Vec<syn::Ident>,
}

// sqlo_set{ "sqlo_as_json", &pool, instance, arg=value,arg=value}
impl syn::parse::Parse for SqloSetParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sqlo_struct_string = input.parse::<syn::LitStr>()?.value();
        input.parse::<Token!(,)>()?;
        let pool = input.parse::<syn::Expr>()?;
        input.parse::<Token!(,)>()?;
        let instance: syn::Ident = input.parse()?;
        input.parse::<Token!(,)>()?;
        let args = syn::punctuated::Punctuated::<syn::Expr, Token!(,)>::parse_terminated(input)?;

        let sqlo: Sqlo = serde_json::from_str(&sqlo_struct_string)
            .map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))?;

        let mut parse_fields: Vec<syn::Ident> = vec![];
        let mut parse_values = vec![];
        for exp in args.into_iter() {
            if let syn::Expr::Assign(exp) = exp {
                let syn::ExprAssign { left, right, .. } = exp;
                if let syn::Expr::Type(syn::ExprType { expr, .. }) = *left {
                    if let syn::Expr::Path(syn::ExprPath { path, .. }) = *expr {
                        if let Some(ident) = path.get_ident() {
                            parse_fields.push(ident.clone());
                            parse_values.push(*right);
                        }
                    }
                }
            }
        }

        Ok(Self {
            sqlo,
            pool,
            instance,
            parse_values,
            parse_fields,
        })
    }
}

impl SqloSetParse {
    pub fn expand(&self) -> TokenStream {
        let SqloSetParse {
            sqlo,
            instance,
            pool,
            parse_values,
            parse_fields,
        } = self;
        let sqlo_ident = &sqlo.ident;
        let pkfield = &sqlo.pk_field;
        let pkfield_ident = &pkfield.ident;

        let values =
            syn::punctuated::Punctuated::<&syn::Expr, Token!(,)>::from_iter(parse_values.iter());

        let (option_struct_name, option_struct) = sqlo.as_option_struct();

        let sqlx_null_check = sqlo
            .fields
            .iter()
            .map(|x| {
                let ident = x.ident.clone();
                if !crate::utils::is_option(&x.ty) {
                    return quote! {
                    if res.#ident.is_none() {return Err(sqlx::Error::RowNotFound)}};
                }
                return quote! {};
            })
            .collect::<TokenStream>();

        let convert_option_to_value = sqlo
            .fields
            .iter()
            .map(|crate::field::Field { ident, ty, .. }| {
                if crate::utils::is_option(ty) {
                    return quote! {#ident:res.#ident,};
                }
                return quote! {#ident:res.#ident.unwrap(),}; //unwrap ok because check in sqlx_null_check
            })
            .collect::<TokenStream>();

        let query = build_sql_query(&sqlo.database_type, sqlo, parse_fields.as_slice());

        quote! {
                async  {
                    #[derive(Debug)]
                    #option_struct

                    match
                        sqlx::query_as!(#option_struct_name,#query, #values,  #instance.#pkfield_ident).fetch_one(#pool).await
                        {
                            Ok(res) => {
                                #sqlx_null_check
                                #instance; // force move
                                Ok(#sqlo_ident{#convert_option_to_value})
                            },
                            Err(e) => {
                                #instance; // force move
                                Err(e)
                            }
                        }
                }
        }
    }
}

fn build_sql_query(
    database_type: &DatabaseType,
    sqlo: &Sqlo,
    parse_fields: &[syn::Ident],
) -> String {
    let Sqlo {
        tablename,
        fields,
        pk_field,
        ..
    } = sqlo;

    let returning_cols = sqlo.all_columns_as_query();
    let pkfield_column = &pk_field.column;

    let columns_names = fields
        .iter()
        .filter_map(|f| {
            if parse_fields.contains(&&f.ident) {
                Some(f.column.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let columns_qmarks = qmarks_with_col(columns_names.as_slice(), database_type);

    format!(
        r#"UPDATE {tablename} SET {columns_qmarks} WHERE {pkfield_column}=? RETURNING {returning_cols};"#
    )
}

pub fn process_sqlo_set(input: SqloSetParse) -> syn::Result<TokenStream> {
    if input.parse_fields.is_empty() {
        return Ok(quote! {});
    }
    Ok(input.expand())
}
