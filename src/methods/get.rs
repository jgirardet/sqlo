use crate::{sqlo::Sqlo, types::get_function_arg_type};
use proc_macro2::TokenStream;
use quote::quote;
use crate::database::db_ident;

pub fn impl_get(s: &Sqlo) -> TokenStream {
    let Sqlo {
        ident,
        tablename,
        all_columns_as_query,
        ..
    } = s;

    let pk_ty = get_function_arg_type(&s.pk_field.ty);
    let database_type = db_ident();

    let pk_column = &s.pk_field.column;

    let query = format!("SELECT {all_columns_as_query} FROM {tablename} WHERE {pk_column}=?");
    quote! {
            /// Get instance by its PrimaryKey.
            pub async fn get<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(pool: E, id: #pk_ty) -> sqlx::Result<#ident> {
                sqlx::query_as!(#ident, #query, id)
                .fetch_one(pool)
                .await
            }
    }
}
