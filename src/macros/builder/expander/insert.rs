use darling::util::IdentString;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprPath};

use crate::{field::Field, macros::Fetch, sqlo::Sqlo, utils::INSERT_FN_FLAG};

use super::WhichMacro;

pub fn expand_insert(
    fetch: Fetch,
    ident: &IdentString,
    query: String,
    arguments: &[Expr],
    main_sqlo: &Sqlo,
) -> TokenStream {
    match fetch {
        Fetch::Stream => {
            unimplemented!("fetch not supported with insert!")
        }
        Fetch::None => {
            if query.contains(INSERT_FN_FLAG) {
                return expand_with_insert_fn(
                    fetch,
                    ident,
                    query,
                    arguments,
                    main_sqlo,
                    WhichMacro::Query,
                );
            }
            quote::quote! {
                |pool|{
                    sqlx::query!(#query, #(#arguments),*).#fetch(pool)
                }
            }
        }
        _ => {
            if query.contains(INSERT_FN_FLAG) {
                return expand_with_insert_fn(
                    fetch,
                    ident,
                    query,
                    arguments,
                    main_sqlo,
                    WhichMacro::QueryAs,
                );
            }
            quote::quote! {
                |pool|{
                    sqlx::query_as!(#ident, #query, #(#arguments),*).#fetch(pool)
                }
            }
        }
    }
}

fn expand_with_insert_fn(
    fetch: Fetch,
    ident: &IdentString,
    query: String,
    arguments: &[Expr],
    main_sqlo: &Sqlo,
    which_macro: WhichMacro,
) -> TokenStream {
    let Sqlo {
        pk_field: Field {
            insert_fn, column, ..
        },
        ..
    } = main_sqlo;
    let insert_fn = insert_fn.clone().unwrap();
    let insert_fn_toks = quote! {let insert_fn = #insert_fn();};
    let arguments: Vec<Expr> = arguments
        .iter()
        .map(|m| match m {
            Expr::Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    if *ident == INSERT_FN_FLAG {
                        let expr: Expr = syn::parse_quote! {insert_fn};
                        return expr;
                    }
                }
                m.clone()
            }
            _ => m.clone(),
        })
        .collect();
    let query = query.replace(INSERT_FN_FLAG, column);
    match which_macro {
        WhichMacro::QueryAs => quote::quote! {
            |pool|{
                async move {
                    #insert_fn_toks
                    sqlx::query_as!(#ident, #query, #(#arguments),*).#fetch(pool).await
                }
            }
        },
        WhichMacro::Query => quote::quote! {
            |pool|{
                async move {
                    #insert_fn_toks
                    sqlx::query!(#query, #(#arguments),*).#fetch(pool).await
                }
            }
        },
    }
}
