use crate::{
    field::Field,
    methods::{create::impl_create, save::impl_save, get::impl_get},
    sqlo::Sqlo,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn produce(sqlo: &Sqlo) -> TokenStream {
    let ident = sqlo.ident.clone();
    let additional_utils = impl_additional_utils(sqlo);
    let crud_queries = impl_crud_queries(sqlo);
    // let set_macro = impl_update_macro(sqlo);

    quote! {

        // #set_macro

        impl <'c>#ident {
            #additional_utils
            #crud_queries
        }

    }
}

fn impl_crud_queries(sqlo: &Sqlo) -> TokenStream {
    let get = impl_get(sqlo);
    let create = impl_create(sqlo);
    let save = impl_save(sqlo);
    quote!(
            #get
            #create
            #save
    )
}


fn impl_update_macro(s: &Sqlo) -> TokenStream {
    let Sqlo {
        ident,
        tablename,
        pk_field,
        fields,
        ..
    } = s;
    let macro_ident = format_ident!("set_{}", ident);
    let tablename = syn::LitStr::new(tablename, ident.span());
    let pk_field = &pk_field.ident;
    // let mut columns = std::collections::HashMap::new();
    let mut columns = vec![];
    for Field { ident, column, .. } in fields.iter() {
        columns.push(format!("{}:{}", ident, column));
    }
    let columns = columns.join(",");

    quote! {
    #[allow(unused_macros)]
    macro_rules! #macro_ident {
        ($pool:expr, $identa:ident) => ();
        // ($pool:expr, $identa:ident, $($arg:ident=$val:expr),+) => (
            // sqlo::sqlo_set!(#tablename #pk_field , $pool , $identa , #columns, $($arg:ident=$val:expr),+)
            // sqlo::sqlo_set!(#tablename #pk_field , $pool , $identa , #columns, $($arg:ident=$val:expr),+)
        // );
    }
    }
}

fn impl_additional_utils(s: &Sqlo) -> TokenStream {
    let Sqlo {
        ident,
        tablename,
        pk_field,
        ..
    } = s;
    // let ident = ident.to_string();
    let pkident = &pk_field.ident;
    let pkcolumn = &pk_field.column;
    let pk_ty = &pk_field.ty;
    quote! {
        fn tablename() -> String {
            #tablename.to_string()
        }

        fn itablename(&self) -> String {
           #ident::tablename()
        }


        fn pk_column(&self) -> String {
            #pkcolumn.to_string()
        }

        fn pk(&self) -> #pk_ty {
            self.#pkident
        }


    }
}
