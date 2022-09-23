use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::Token;

use crate::{
    query_builder::qmarks_with_col,
    sqlo::{DatabaseType, Sqlo},
    utils::compile_error,
};

pub struct SqloSetParse {
    sqlo: Sqlo,
    iov: InstanceOrValue,
    parse_fields: Vec<syn::Ident>,
    parse_values: Vec<syn::Expr>,
}

// sqlo_set{ "sqlo_as_json" [for instance|where value], arg=value,arg=value}
impl syn::parse::Parse for SqloSetParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // parse sqlo as str
        let sqlo_struct_string = input.parse::<syn::LitStr>()?.value();

        // get instance or value
        let iov = if input.peek(Token![for]) {
            input.parse::<Token!(for)>()?;
            InstanceOrValue::Instance(input.parse::<syn::Ident>()?)
        } else if input.peek(Token![where]) {
            input.parse::<Token!(where)>()?;
            InstanceOrValue::Value(input.parse::<syn::Expr>()?)
        } else {
            return Err(input.error("Only for,where are allowed"));
        };

        // parse args  : fields=value
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
            iov,
            parse_values,
            parse_fields,
        })
    }
}

impl SqloSetParse {
    pub fn expand(&self) -> TokenStream {
        let SqloSetParse {
            sqlo,
            iov,
            parse_values,
            parse_fields,
        } = self;

        let Sqlo { pk_field, .. } = sqlo;

        let pk_value = get_pk_value(iov, &pk_field.ident);
        let drop_instance = if let InstanceOrValue::Instance(ref i) = iov {
            quote![drop(#i);]
        } else {
            quote!()
        };

        let values =
            syn::punctuated::Punctuated::<&syn::Expr, Token!(,)>::from_iter(parse_values.iter());

        let (option_struct_name, option_struct) = sqlo.as_option_struct();
        let (sqlx_null_check, converted_from_option_struct) =
            sqlo.convert_struct_option_to_struct();

        let query = build_sql_query(&sqlo.database_type, sqlo, parse_fields.as_slice());

        quote! {
            |pool|
                {
                async move  {
                    #[derive(Debug)]
                    #option_struct

                    match
                        sqlx::query_as!(#option_struct_name,#query, #values,  #pk_value).fetch_one(pool).await
                        {
                            Ok(res) => {
                                #sqlx_null_check
                                #drop_instance //force move
                                Ok(#converted_from_option_struct)
                            },
                            Err(e) => {
                                #drop_instance // force move
                                Err(e)
                            }
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

#[derive(Debug)]
enum InstanceOrValue {
    Instance(syn::Ident),
    Value(syn::Expr),
}

fn get_pk_value(iov: &InstanceOrValue, pk_field: &syn::Ident) -> TokenStream {
    match *iov {
        InstanceOrValue::Instance(ref ident) => quote![#ident.#pk_field],
        InstanceOrValue::Value(ref expr_value) => match expr_value {
            syn::Expr::Lit(expr_lit) => expr_lit.to_token_stream(),
            syn::Expr::Path(expr_path) => expr_path.to_token_stream(),
            syn::Expr::Index(expr_index) => expr_index.to_token_stream(),
            syn::Expr::Field(expr_field) => expr_field.to_token_stream(),
            _ => compile_error(
                expr_value,
                "Unsupported Expression: use identifier, field, index or literal",
            ),
        },
    }
}

pub fn impl_update_macro(s: &Sqlo) -> TokenStream {
    let Sqlo { ident, fields, .. } = s;

    if fields.len() == 1 {
        return quote! {}; // no macro if only pk is set for struct
    }

    let macro_ident = format_ident!("update_{}", ident);
    let sqlo_struct = serde_json::to_string(&s).expect("Fail serializing Sqlo to json");

    quote! {
    #[allow(unused_macros)]
    macro_rules! #macro_ident {
        // by instance
        ($instance:ident; $($arg:ident=$val:expr),+) => (
            sqlo::sqlo_set!(#sqlo_struct for $instance, $($arg:ident=$val:expr),+)
        );
        // by varname, literal
        (pk $pk_value:tt ; $($arg:ident=$val:expr),+) => (
            sqlo::sqlo_set!(#sqlo_struct where $pk_value, $($arg:ident=$val:expr),+)
        );
        // by index
        (pk $pk_value:tt[$num:literal] ; $($arg:ident=$val:expr),+) => (
            sqlo::sqlo_set!(#sqlo_struct where $pk_value[$num], $($arg:ident=$val:expr),+)
        );
        // by field
        (pk $pk_value:tt$(.$field:tt)+ ; $($arg:ident=$val:expr),+) => (
            sqlo::sqlo_set!(#sqlo_struct where $pk_value$(.$field)+, $($arg:ident=$val:expr),+)
        );
    }
    }
}
pub fn process_sqlo_set(input: SqloSetParse) -> syn::Result<TokenStream> {
    if input.parse_fields.is_empty() {
        return Ok(quote! {});
    }
    Ok(input.expand())
}
