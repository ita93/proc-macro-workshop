extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bname = format!("{}Builder",name);
    let fields = if let Data::Struct(ds) = &ast.data {
        ds.fields.clone()
    } else {
        unimplemented!()
    };

    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote!{
            #name: std::option::Option<#ty>
        }
    });

    let fnized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        quote!{
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
            /*fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
            
            fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }

            fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }

            fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }
            */
            #(#fnized)*

            pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {
                Ok(
                    #name {
                        executable: self.executable.clone().ok_or("Missing executable argument")?,
                        args: self.args.clone().ok_or("Missing args argument")?,
                        env: self.env.clone().ok_or("Missing env argument")?,
                        current_dir: self.current_dir.clone().ok_or("Missing current_dir argument")?,
                    }
                )    
            }    
        }
    };
    expaned.into()
}
