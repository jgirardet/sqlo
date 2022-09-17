use proc_macro2::TokenStream;
use quote::quote;
use crate::sqlo::Sqlo;


// Entity.get(id)
pub fn impl_get(s: &Sqlo) -> TokenStream {
    let Sqlo {
        ident,
        tablename,
        database_type,
        ..
    } = s;
    let all_columns_as_query = s.all_columns_as_query();
    let pk_ty = &s.pk_field.ty;
    let pk_column = &s.pk_field.column;
    let query = format!("SELECT {all_columns_as_query} FROM {tablename} WHERE {pk_column}=?");
    quote! {
            /// Get instance by its PrimaryKey.
            async fn get<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(pool: E, id: #pk_ty) -> sqlx::Result<#ident> {
                sqlx::query_as!(#ident, #query, id)
                .fetch_one(pool)
                .await
            }
    }
}