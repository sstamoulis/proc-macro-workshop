use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &derive_input.ident;
    let syn::Data::Struct(data) = &derive_input.data else {unimplemented!()};
    let fields_ident: Vec<syn::Ident> = data
        .fields
        .iter()
        .map(|f| f.ident.clone().unwrap_or_else(|| unimplemented!()))
        .collect();
    let fields_type: Vec<_> = data.fields.iter().map(|f| f.ty.clone()).collect();
    let fields_optional: Vec<bool> = data
        .fields
        .iter()
        .map(|f| match &f.ty {
            syn::Type::Path(type_path) => type_path.path.is_ident("Option"),
            _ => unimplemented!(),
        })
        .collect();
    let builder_struct_ident = format_ident!("{}Builder", derive_input.ident);
    let builder_struct = {
        let fields = data.fields.iter().map(|f| {
            let Some(ident) = &f.ident else {unimplemented!()};
            let syn::Type::Path(ty) = &f.ty else {unimplemented!()};
            quote!(#ident: Option<#ty>)
        });
        quote! {
            pub struct #builder_struct_ident {
                #(#fields),*
            }
        }
    };

    let builder_constructor = {
        let fields_init = match &derive_input.data {
            syn::Data::Struct(data) => data.fields.iter().map(|f| {
                let Some(ident) = &f.ident else {unimplemented!()};
                quote!(#ident: None)
            }),
            _ => unimplemented!(),
        };
        quote! {
            impl #struct_ident {
                pub fn builder() -> #builder_struct_ident {
                    #builder_struct_ident {
                        #(#fields_init),*
                    }
                }
            }
        }
    };

    let builder_methods = {
        let types = data.fields.iter().map(|f| &f.ty);
        quote! {
            #(
                fn #fields_ident(&mut self, value: #types) -> &mut Self {
                    self.#fields_ident = Some(value);
                    self
                }
            )*
        }
    };

    let builder_build_method = {
        let errors = fields_ident
            .iter()
            .map(|f| format!("Required argument {} is missing", f));
        quote! {
            pub fn build(&mut self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
                Ok(#struct_ident {
                    #(#fields_ident: self.#fields_ident.clone().ok_or(#errors)?),*
                })
            }
        }
    };

    quote! {
        #builder_constructor
        #builder_struct
        impl #builder_struct_ident {
            #builder_methods
            #builder_build_method
        }
    }
    .into()
}
