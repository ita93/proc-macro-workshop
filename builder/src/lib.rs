extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Attribute, Data, DeriveInput, PathArguments,
};

fn is_an_option(f: &syn::Field) -> Option<AngleBracketedGenericArguments> {
    let ty = &f.ty;

    match ty {
        syn::Type::Path(val) => {
            let ty_ident = val.path.segments.first().unwrap().ident.to_string();
            if ty_ident == "Option" {
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

fn has_attribute(f: &syn::Field) -> Option<String> {
    match f.attrs.first() {
        Some(attr) => {
            match attr.parse_meta() {
                Ok(syn::Meta::List(mtl)) => {
                    let attr_path = mtl.path.get_ident().map_or(String::from("undifined"), |id| id.to_string());
                    assert_eq!(&attr_path, "builder"); //only accept builder
                    //only accept builder
                    match mtl.nested.first() {
                        Some(syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name))) => {
                            if &meta_name.path.get_ident().map_or(String::from("undefined"), |id| id.to_string()) != "each" {
                                return None;
                            }
                            
                            if let syn::Lit::Str(lit_str) = &meta_name.lit {
                                return Some(lit_str.value());
                            }
                        }
                        _ => {
                            println!("Cannot find meta_name");
                        }
                    }
                }
                _ => {
                    println!("MEta list not found");
                }
            }
        }
        None => {
            return None;
        }
    }

    None
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    //    println!("{:#?}", &ast);
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

    let build_fn = fields.iter().map(|f| {
        let name = &f.ident;

        match is_an_option(f) {
            Some(_) => {
                quote! {
                    #name: self.#name.clone()
                }
            }
            None => {
                quote! {
                    #name: self.#name.clone().ok_or(concat!("Missing ", stringify!(#name), " argument"))?
                }
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
                        #(#build_fn,)*
                    }
                )
            }
        }
    };
    expaned.into()
}
