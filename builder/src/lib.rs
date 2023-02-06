mod field;
use field::{Each, GetIdent, Optional};
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &derive_input.ident;
    let syn::Data::Struct(data) = &derive_input.data else {unimplemented!()};

    let builder_struct_ident = format_ident!("{}Builder", derive_input.ident);
    let builder_struct = {
        let (idents, types): (Vec<_>, Vec<_>) = data
            .fields
            .iter()
            .map(|f| (f.get_ident(), &f.ty))
            .multiunzip();
        quote! {
            pub struct #builder_struct_ident {
                #(#idents: Option<#types>),*
            }
        }
    };

    let builder_constructor = {
        let idents = data.fields.iter().map(|f| f.get_ident());
        quote! {
            impl #struct_ident {
                pub fn builder() -> #builder_struct_ident {
                    #builder_struct_ident {
                        #(#idents: None),*
                    }
                }
            }
        }
    };

    let builder_methods = {
        quote!()
        // let methods =
        //     multizip((&fields_ident, &fields_type, &fields_each)).map(|(ident, ty, each)| {
        //         let method_one = match each {
        //             Some(each) => match ty.path.segments.iter().exactly_one() {
        //                 Ok(segment) if segment.ident == "Vec" => match &segment.arguments {
        //                     syn::PathArguments::AngleBracketed(
        //                         syn::AngleBracketedGenericArguments { args, .. },
        //                     ) => match args.iter().exactly_one() {
        //                         Ok(syn::GenericArgument::Type(ty)) => quote! {
        //                             fn #each(&mut self, value: #ty) -> &mut Self {
        //                                 match &mut self.#ident {
        //                                     Some(v) => v.push(value),
        //                                     None => {
        //                                         let mut v = Vec::new();
        //                                         v.push(value);
        //                                         self.#ident = Some(v);
        //                                     }
        //                                 }
        //                                 self
        //                             }
        //                         },
        //                         _ => unimplemented!(),
        //                     },
        //                     _ => unimplemented!(),
        //                 },
        //                 _ => unimplemented!(),
        //             },
        //             None => quote!(),
        //         };

        //         let method_all = match each {
        //             Some(each) if each == ident => quote!(),
        //             _ => quote! {
        //                 fn #ident(&mut self, value: #ty) -> &mut Self {
        //                     self.#ident = Some(value);
        //                     self
        //                 }
        //             },
        //         };
        //         quote! {
        //             #method_all
        //             #method_one
        //         }
        //     });
        // quote!(#(#methods)*)
        // quote! {
        //     #(
        //         fn #fields_ident(&mut self, value: #fields_type) -> &mut Self {
        //             self.#fields_ident = Some(value);
        //             self
        //         }
        //     )*
        // }
    };

    let builder_build_method = {
        let idents = data.fields.iter().map(|f| f.get_ident());
        let error_handlers = data.fields.iter().map(|f| {
            if let Some(_) = f.get_optional_type() {
                let ident = f.get_ident();
                let message = format!("Required argument {} is missing", ident);
                quote!(.ok_or(#message)?)
            } else {
                quote!()
            }
        });
        quote! {
            pub fn build(&mut self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
                Ok(#struct_ident {
                    #(#idents: self.#idents.clone()#error_handlers),*
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
