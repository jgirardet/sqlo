use proc_macro2::TokenStream;

use crate::{
    query_builder::{commma_sep_with_parenthes_literal_list, qmarks},
    sqlo::{DatabaseType, Sqlo},
    types::get_function_arg_type,
};
use quote::quote;

struct CrudCreateImpl<'a> {
    pub non_create_fn_idents: Vec<&'a syn::Ident>,
    pub non_create_fn_types: Vec<&'a syn::Type>,
    pub create_fn_idents: Vec<&'a syn::Ident>,
    pub create_fns: Vec<&'a syn::ExprPath>,
    pub insert_query_columns: Vec<&'a str>,
    pub insert_query_args: Vec<&'a syn::Ident>,
}

impl<'a> From<&'a Sqlo> for CrudCreateImpl<'a> {
    fn from(s: &'a Sqlo) -> CrudCreateImpl {
        let mut non_create_fn_idents = vec![];
        let mut non_create_fn_types = vec![];
        let mut create_fn_idents = vec![];
        let mut create_fns = vec![];

        let mut insert_query_args = vec![];
        let mut insert_query_columns = vec![];

        for f in &s.fields {
            if f == &s.pk_field {
                if let Some(ref func) = f.create_fn {
                    create_fn_idents.push(&f.ident);
                    create_fns.push(func);
                } else if f.create_arg {
                    non_create_fn_idents.push(&f.ident);
                    non_create_fn_types.push(&f.ty);
                } else {
                    continue;
                }
            } else {
                non_create_fn_idents.push(&f.ident);
                non_create_fn_types.push(&f.ty);
            }
            insert_query_columns.push(f.column.as_str());
            insert_query_args.push(&f.ident)
        }

        CrudCreateImpl {
            non_create_fn_idents,
            non_create_fn_types,
            create_fn_idents,
            create_fns,
            insert_query_columns,
            insert_query_args,
        }
    }
}
// Entity.create(id)
pub fn impl_create(s: &Sqlo) -> TokenStream {
    let Sqlo {
        ident,
        tablename,
        database_type,
        ..
    } = s;

    let all_columns_as_query = s.all_columns_as_query();
    let CrudCreateImpl {
        non_create_fn_idents,
        non_create_fn_types,
        create_fn_idents,
        create_fns,
        insert_query_columns,
        insert_query_args,
    } = s.into();

    let insert_query_args = quote! { #(#insert_query_args),*};
    let non_create_fn_types: Vec<TokenStream> = non_create_fn_types
        .into_iter()
        .map(|x| get_function_arg_type(x))
        .collect();
    let fn_args = quote! {#(#non_create_fn_idents:#non_create_fn_types),*};

    let create_fn_impl = quote! {
        #(let #create_fn_idents = #create_fns();)*
    };

    let (option_struct_name, option_struct) = s.as_option_struct();

    let (sqlx_null_check, converted_from_option_struct) = s.convert_struct_option_to_struct();

    let query = build_sql_query(
        &s.database_type,
        &tablename,
        &insert_query_columns,
        &all_columns_as_query,
    );
    quote! {

            /// Creates a new entry in database and returns the new instance.
            ///
            /// Every field is used as argument by default in their declaring order except PrimaryKey.
            /// Use attribute `creat_fn` to delegate value  to a function. ex : #[sqlo(create_fn="uuid::Uuid::new_v4")]
            /// Use `create_arg` with PrimaryKey to add it with other input arguments.
            async fn create<E: sqlx::Executor<'c, Database = sqlx::#database_type>>(pool: E, #fn_args) -> sqlx::Result<#ident> {
                #option_struct
                #create_fn_impl

                let res = sqlx::query_as!(#option_struct_name, #query, #insert_query_args)
                .fetch_one(pool)
                .await?;

                #sqlx_null_check
                Ok(#converted_from_option_struct)
            }
    }
}

fn build_sql_query(
    database_type: &DatabaseType,
    tablename: &str,
    set_columns_names: &[&str],
    returnin_columns: &str,
) -> String {
    let mut qmarks = qmarks(set_columns_names.len(), &database_type);
    if qmarks == "" {
        qmarks = "NULL".to_string();
    }

    let columns = commma_sep_with_parenthes_literal_list(set_columns_names);

    format!("INSERT INTO {tablename} {columns} VALUES({qmarks}) RETURNING {returnin_columns};")
}

#[cfg(test)]
#[allow(non_snake_case)]
mod crud_create {
    use super::*;
    macro_rules! test_create_build_query {
        ($db:tt, $titre:literal, [$($cols:literal),*], $res:literal) => {
            paste::paste! {
                #[test]
                fn [<create_query_builder_ $db _ $titre>]() {
                    assert_eq!(build_sql_query($db, "bla", &[$(&$cols),*], "ret,col"), $res)
                }
            }
        };
    }

    const SQLITE: &DatabaseType = &DatabaseType::Sqlite;
    test_create_build_query!(
        SQLITE,
        "no_arg",
        [],
        "INSERT INTO bla  VALUES(NULL) RETURNING ret,col;"
    );
    test_create_build_query!(
        SQLITE,
        "un_arg",
        ["un"],
        "INSERT INTO bla (un) VALUES(?) RETURNING ret,col;"
    );
    test_create_build_query!(
        SQLITE,
        "deux_arg",
        ["un", "deux"],
        "INSERT INTO bla (un,deux) VALUES(?,?) RETURNING ret,col;"
    );
}
