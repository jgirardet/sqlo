use proc_macro2::TokenStream;
use quote::quote;

use crate::{field::Field, query_builder::qmarks, sqlo::Sqlo, types::get_function_arg_type};

pub fn impl_delete(sqlo: &Sqlo) -> TokenStream {
    let Sqlo {
        database_type,
        pk_field,
        ..
    } = sqlo;
    let Field {
        ident: pk_ident,
        ty: pk_type,
        ..
    } = pk_field;
    let query = build_sql_query(sqlo);

    let pk_type = get_function_arg_type(pk_type);

    quote![

        /// Delete database row.
        /// Instance is moved preventing any other use.
        async fn remove<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(self, pool: E) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult> {
            sqlx::query!(#query, self.#pk_ident ).execute(pool).await
        }

        /// Delete database row using primary_key
        async fn delete<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(pool: E, pk: #pk_type ) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult> {
            sqlx::query!(#query, pk ).execute(pool).await
        }
    ]
}

fn build_sql_query(sqlo: &Sqlo) -> String {
    let Sqlo {
        tablename,
        database_type,
        pk_field,
        ..
    } = sqlo;
    let qmark = qmarks(1, database_type);
    let pk_column = &pk_field.column;
    format!("DELETE FROM {tablename} WHERE {pk_column}={qmark};")
}
