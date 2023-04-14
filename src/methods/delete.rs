use proc_macro2::TokenStream;
use quote::quote;

use crate::database::{db_ident, db_query_result_path, db_sqlx_path, qmarks};
use crate::{field::Field, sqlo::Sqlo, types::get_function_arg_type};

pub fn impl_delete(sqlo: &Sqlo) -> TokenStream {
    let Sqlo { pk_field, .. } = sqlo;
    let Field {
        ident: pk_ident,
        ty: pk_type,
        ..
    } = pk_field;
    let query = build_sql_query(sqlo);

    let pk_type = get_function_arg_type(pk_type);

    let database_type = db_ident();
    let db_path = db_sqlx_path();
    let sqlx_qr_path = db_query_result_path();

    quote![

        /// Delete database row.
        /// Instance is moved preventing any other use.
        pub async fn remove<E: sqlx::Executor<'c, Database = #db_path>>(self, pool: E) -> sqlx::Result<#sqlx_qr_path> {
            sqlx::query!(#query, self.#pk_ident ).execute(pool).await
        }

        /// Delete database row using primary_key
        pub async fn delete<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(pool: E, pk: #pk_type ) -> sqlx::Result<#sqlx_qr_path> {
            sqlx::query!(#query, pk ).execute(pool).await
        }
    ]
}

fn build_sql_query(sqlo: &Sqlo) -> String {
    let Sqlo {
        tablename,
        pk_field,
        ..
    } = sqlo;
    let qmark = qmarks(1);
    let pk_column = &pk_field.column;
    format!("DELETE FROM {tablename} WHERE {pk_column}={qmark};")
}
