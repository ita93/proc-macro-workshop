extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, PathArguments};

fn is_an_option(f: &syn::Field) -> Option<AngleBracketedGenericArguments> {
    let ty = &f.ty;

    match ty {
        syn::Type::Path(val) => {
            let ty_ident = val.path.segments.first().unwrap().ident.clone();
            if ty_ident.to_string() == "Option" {
                if let PathArguments::AngleBracketed(angle_generic) =
                    &val.path.segments.first().unwrap().arguments
                {
                    return Some(angle_generic.clone());
                };
            }
            None
        }
        _ => None,
    }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bname = format!("{}Builder", name);
    let fields = if let Data::Struct(ds) = &ast.data {
        ds.fields.clone()
    } else {
        unimplemented!()
    };

    // If the Fields input is Option then don't need to wrap it
    // in another Option level.
    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty; //This variable is a Type

        match is_an_option(f) {
            Some(generic_arg) => {
                quote! {
                    #name: std::option::Option#generic_arg
                }
            }
            None => {
                quote! {
                    #name: std::option::Option<#ty>
                }
            }
        }
    });

    let fnized = fields.iter().map(|f| {
        let name = &f.ident;
        let mut ty = &f.ty;

        let is_option = is_an_option(f);

        if let Some(ref generic_arg) = is_option {
            if let syn::GenericArgument::Type(ref base_ty) = generic_arg.args.first().unwrap() {
                ty = base_ty;
            }
        }
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self{
                self.#name = Some(#name);
                self
            }
        }
    });

    let bident = syn::Ident::new(&bname, name.span());
    let expaned = quote! {
        use std::error::Error;

        struct #bident {
            #(#optionized,)*
        }

        impl #name {
            fn builder() -> #bident {
                #bident{
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        impl #bident{
            #(#fnized)*

            pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {
                Ok(
                    #name {
                        executable: self.executable.clone().ok_or("Missing executable argument")?,
                        args: self.args.clone().ok_or("Missing args argument")?,
                        env: self.env.clone().ok_or("Missing env argument")?,
                        current_dir: self.current_dir.clone(),
                    }
                )
            }
        }
    };
    expaned.into()
}
