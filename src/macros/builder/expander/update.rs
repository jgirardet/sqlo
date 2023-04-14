use crate::macros::Fetch;
use darling::util::IdentString;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn expand_update(
    fetch: Fetch,
    ident: &IdentString,
    query: String,
    arguments: &[&Expr],
    move_instance: TokenStream,
) -> TokenStream {
    match fetch {
        Fetch::Stream => {
            quote! {
                |pool|{
                    sqlx::query_as!(#ident,#query, #(#arguments),*).#fetch(pool)
                }
            }
        }
        Fetch::None => {
            quote::quote! {

                |pool|{
                    async move {
                    #move_instance
                    sqlx::query!(#query, #(#arguments),*).#fetch(pool).await
                    }
                }
            }
        }
        _ => {
            quote::quote! {
                |pool|{
                    async move {
                    #move_instance
                    sqlx::query_as!(#ident, #query, #(#arguments),*).#fetch(pool).await
                    }
                }
            }
        }
    }
}
