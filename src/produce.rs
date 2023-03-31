use crate::{
    methods::{delete::impl_delete, get::impl_get, save::impl_save},
    sqlo::Sqlo,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn produce(sqlo: &Sqlo) -> TokenStream {
    let ident = sqlo.ident.clone();
    let additional_utils = impl_additional_utils(sqlo);
    let crud_queries = impl_crud_queries(sqlo);

    quote! {
        impl <'c>#ident {
            #additional_utils
            #crud_queries
        }

    }
}

fn impl_crud_queries(sqlo: &Sqlo) -> TokenStream {
    let get = impl_get(sqlo);
    let save = impl_save(sqlo);
    let delete = impl_delete(sqlo);
    quote!(
            #get
            #save
            #delete
    )
}

fn impl_additional_utils(s: &Sqlo) -> TokenStream {
    let Sqlo {
        ident, pk_field, ..
    } = s;
    let pkident = &pk_field.ident;
    let pk_ty = &pk_field.ty;
    let ident_name = LitStr::new(ident.as_str(), ident.span());
    quote! {
        pub fn pk(&self) -> &#pk_ty {
            &self.#pkident
        }

        pub fn sqlo_struct_name(&self) -> &str {
            #ident_name
        }


    }
}
