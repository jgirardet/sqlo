use crate::{
    database::{db_ident, qmarks, qmarks_with_col},
    field::Field,
    sqlo::Sqlo,
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_save(sqlo: &Sqlo) -> TokenStream {
    let Sqlo {
        tablename,
        pk_field,
        fields,
        ..
    } = sqlo;

    let q_columns: Vec<&str> = fields.iter().map(|x| x.column.as_str()).collect();
    let columns_no_pk = q_columns
        .iter()
        .filter(|c| c != &&pk_field.column.as_str())
        .copied()
        .collect::<Vec<_>>();

    let idents = fields.iter().map(|f| f.ident.clone()).collect::<Vec<_>>();
    let update_fields = fields
        .iter()
        .filter(|Field { ident, .. }| ident != &pk_field.ident)
        .map(|f| &f.ident);

    let self_fields = idents.iter().chain(update_fields);
    let q_self_fields = quote! {#(self.#self_fields),*};

    let query = build_sql_query(
        tablename,
        &q_columns,
        &pk_field.column,
        columns_no_pk.as_slice(),
    );

    let database_type = db_ident();

    quote! {

            /// Create a new row with instance or update it if pk already exists.
            ///
            /// It's an UPSERT statement based  on Primary Key.
            pub async fn save<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(&self, pool: E) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult> {
                sqlx::query!(#query, #q_self_fields )
                .execute(pool)
                .await

            }
    }
}

fn build_sql_query(
    tablename: &str,
    columns_array: &[&str],
    pk_column: &str,
    col_if_update: &[&str],
) -> String {
    let mut qmarks = qmarks(columns_array.len());
    if qmarks.is_empty() {
        qmarks = "NULL".to_string();
    }
    let col_qmarks_if_update = qmarks_with_col(col_if_update);

    let on_conflict = if columns_array.len() > 1 {
        format!("DO UPDATE SET {col_qmarks_if_update}")
    } else {
        "DO NOTHING".to_string() //no update if pk exists and is the only field
    };

    let columns = commma_sep_with_parenthes_literal_list(columns_array);
    // let columns = if columns_array.is_empty() {
    //     String::new()
    // } else {
    //     format!(
    //         "({})",
    //         columns_array
    //             .iter()
    //             .fold(String::new(), |acc, nex| format!("{},{}", acc, nex))
    //     )
    // };

    format!("INSERT INTO {tablename} {columns} VALUES({qmarks}) ON CONFLICT ({pk_column}) {on_conflict};")
}
fn commma_sep_with_parenthes_literal_list(list: &[&str]) -> String {
    if list.is_empty() {
        return "".to_string();
    }
    let sep_comad = list.iter().join(",");
    format!("({sep_comad})")
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
#[allow(non_snake_case)]
mod crud_save {
    use super::*;
    #[test]
    fn test_save_sql_args_query_builder() {
        assert_eq!(build_sql_query("latable", &["un","deux"], "lepk", &["col","if","update"]), 
        "INSERT INTO latable (un,deux) VALUES(?,?) ON CONFLICT (lepk) DO UPDATE SET col=?,if=?,update=?;")
    }
    macro_rules! test_save_build_query {
        ($titre:literal, [$($cols:literal),*], $res:literal) => {
            paste::paste! {
                #[test]
                fn [<save_query_builder _ $titre>]() {
                    assert_eq!(build_sql_query("bla", &[$(&$cols),*], "pk", &["set","col"]), $res)
                }
            }
        };
    }

    test_save_build_query!(
        // sould not be possible I think
        "no_arg",
        [],
        "INSERT INTO bla  VALUES(NULL) ON CONFLICT (pk) DO NOTHING;"
    );
    test_save_build_query!(
        "un_arg",
        ["pk"],
        "INSERT INTO bla (pk) VALUES(?) ON CONFLICT (pk) DO NOTHING;"
    );
    test_save_build_query!(
        "deux_arg",
        ["pk", "deux"],
        "INSERT INTO bla (pk,deux) VALUES(?,?) ON CONFLICT (pk) DO UPDATE SET set=?,col=?;"
    );

    use super::commma_sep_with_parenthes_literal_list;

    #[test]
    fn is_empty() {
        assert_eq!(commma_sep_with_parenthes_literal_list(&[]), "")
    }
    #[test]
    fn is_not_empty() {
        assert_eq!(
            commma_sep_with_parenthes_literal_list(&["bla", "bli"]),
            "(bla,bli)"
        )
    }
}
