extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bname = format!("{}Builder",name);
    let bident = syn::Ident::new(&bname, name.span());
    let expaned = quote! {
        struct #bident {

        }

        impl #name {
            fn builder() -> #bident {
                #bident
            }
        }
    };
    expaned.into()
}
