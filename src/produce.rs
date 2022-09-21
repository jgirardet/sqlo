use crate::{
    methods::{create::impl_create, get::impl_get, save::impl_save},
    sqlo::Sqlo,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn produce(sqlo: &Sqlo) -> TokenStream {
    let ident = sqlo.ident.clone();
    let additional_utils = impl_additional_utils(sqlo);
    let crud_queries = impl_crud_queries(sqlo);
    let set_macro = impl_update_macro(sqlo);

    quote! {

        #set_macro

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
    let Sqlo { ident, fields, .. } = s;

    if fields.len() == 1 {
        return quote! {}; // no macro if only pk is set for struct
    }

    let macro_ident = format_ident!("set_{}", ident);
    let sqlo_struct = serde_json::to_string(&s).expect("Fail serializing Sqlo to json");

    quote! {
    #[allow(unused_macros)]
    macro_rules! #macro_ident {
        ($pool:expr, $instance:ident, $($arg:ident=$val:expr),+) => (
            sqlo::sqlo_set!(#sqlo_struct, $pool , $instance , $($arg:ident=$val:expr),+)
        );
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
