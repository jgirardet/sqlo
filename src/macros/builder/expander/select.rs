use crate::macros::Fetch;
use darling::util::IdentString;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use super::WhichMacro;

pub fn expand_select(
    fetch: Fetch,
    ident: &IdentString,
    query: String,
    arguments: &[&Expr],
    wich_macro: WhichMacro,
) -> TokenStream {
    match fetch {
        Fetch::Stream => {
            quote! {
                |pool|{
                    sqlx::query_as!(#ident,#query, #(#arguments),*).#fetch(pool)
                }
            }
        }
        _ => {
            if let WhichMacro::Query = wich_macro {
                quote::quote! {
                    |pool|
                        {
                            sqlx::query!(#query, #(#arguments),*).#fetch(pool)
                        }
                }
            } else {
                quote::quote! {
                    |pool|
                         {
                        sqlx::query_as!(#ident, #query, #(#arguments),*).#fetch(pool)
                        }

                }
            }
        }
    }
}
