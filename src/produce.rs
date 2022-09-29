use crate::{
    methods::{create::impl_create, delete::impl_delete, get::impl_get, save::impl_save},
    sqlo::Sqlo,
    sqlo_update::impl_update_macro,
};
use proc_macro2::TokenStream;
use quote::quote;

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
    let delete = impl_delete(sqlo);
    quote!(
            #get
            #create
            #save
            #delete
    )
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
            self.#pkident.clone()
        }


    }
}
