extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, PathArguments};

enum InnerType {
    VecInner(AngleBracketedGenericArguments),
    OptionInner(AngleBracketedGenericArguments),
    None,
}

fn get_inner(f: &syn::Field) -> InnerType {
    let ty = &f.ty;

    match ty {
        syn::Type::Path(val) => {
            let ty_ident = val.path.segments.first().unwrap().ident.to_string();
            if let PathArguments::AngleBracketed(angle_generic) =
                &val.path.segments.first().unwrap().arguments
            {
                if ty_ident == "Option" {
                    return InnerType::OptionInner(angle_generic.clone());
                } else if ty_ident == "Vec" {
                    return InnerType::VecInner(angle_generic.clone());
                }
            };
            InnerType::None
        }
        _ => InnerType::None,
    }
}

fn has_attribute(f: &syn::Field) -> Option<String> {
    match f.attrs.first() {
        Some(attr) => {
            match attr.parse_meta() {
                Ok(syn::Meta::List(mtl)) => {
                    let attr_path = mtl
                        .path
                        .get_ident()
                        .map_or(String::from("undifined"), |id| id.to_string());
                    assert_eq!(&attr_path, "builder"); //only accept builder
                                                       //only accept builder
                    match mtl.nested.first() {
                        Some(syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name))) => {
                            if &meta_name
                                .path
                                .get_ident()
                                .map_or(String::from("undefined"), |id| id.to_string())
                                != "each"
                            {
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

        match get_inner(f) {
            InnerType::OptionInner(generic_arg) => {
                quote! {
                    #name: std::option::Option#generic_arg
                }
            }
            _ => {
                quote! {
                    #name: std::option::Option<#ty>
                }
            }
        }
    });

    let fnized = fields.iter().map(|f| {
        let name = &f.ident;
        let mut ty = &f.ty;

        let raw_name = if let Some(id) = &name {
            id.to_string()
        } else {
            String::from("noname")
        };

        let is_option = get_inner(f);

        if let InnerType::OptionInner(ref generic_arg) = is_option {
            if let syn::GenericArgument::Type(ref base_ty) = generic_arg.args.first().unwrap() {
                ty = base_ty;
            }
        } else if let InnerType::VecInner(ref generic_arg) = is_option {
            if let syn::GenericArgument::Type(ref base_ty) = generic_arg.args.first().unwrap() {
                if let Some(inner_name) = has_attribute(f) {
                    let inner_id = syn::Ident::new(&inner_name, proc_macro2::Span::call_site());
                    if inner_name != raw_name {
                        return quote! {
                            pub fn #inner_id(&mut self, #name: #base_ty) -> &mut Self{
                                let mut vec_values = self.#name.get_or_insert(Vec::new());
                                vec_values.push(#name);
                                self
                            }

                            pub fn #name(&mut self, #name: #ty) -> &mut Self{
                                self.#name = Some(#name);
                                self
                            }
                        };
                    } else {
                        return quote! {
                            pub fn #inner_id(&mut self, #name: #base_ty) -> &mut Self{
                                let mut vec_values = self.#name.get_or_insert(Vec::new());
                                vec_values.push(#name);
                                self
                            }
                        };
                    }
                }
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

        match get_inner(f) {
            InnerType::OptionInner(_) => {
                quote! {
                    #name: self.#name.clone()
                }
            }
            _ => {
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
                    args: Some(Vec::new()),
                    env: Some(Vec::new()),
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
