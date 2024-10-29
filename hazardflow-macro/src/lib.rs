//! HazardFlow macros

use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput, Item, ItemFn};

#[proc_macro_attribute]
pub fn synthesize(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    f.attrs.push(parse_quote!(#[hazardflow::synthesize]));
    f.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn magic(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = args.to_string();
    let item = parse_macro_input!(item as Item);

    match item {
        Item::Const(_) => todo!(),
        Item::Enum(_) => todo!(),
        Item::ExternCrate(_) => todo!(),
        Item::Fn(mut f) => {
            f.attrs.push(parse_quote!(#[hazardflow::magic(#args)]));
            f.into_token_stream().into()
        }
        Item::ForeignMod(_) => todo!(),
        Item::Impl(mut imp) => {
            imp.attrs.push(parse_quote!(#[hazardflow::magic(#args)]));
            imp.into_token_stream().into()
        }
        Item::Macro(_) => todo!(),
        Item::Mod(_) => todo!(),
        Item::Static(_) => todo!(),
        Item::Struct(mut s) => {
            s.attrs.push(parse_quote!(#[hazardflow::magic(#args)]));
            s.into_token_stream().into()
        }
        Item::Trait(_) => todo!(),
        Item::TraitAlias(_) => todo!(),
        Item::Type(_) => todo!(),
        Item::Union(_) => todo!(),
        Item::Use(_) => todo!(),
        Item::Verbatim(_) => todo!(),
        _ => todo!(),
    }
}

#[proc_macro_derive(HEq)]
pub fn heq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let name = &ast.ident;
    match ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }), ..
        }) => {
            let fields = named;

            assert_ne!(named.len(), 0);

            let fs = fields.iter().map(|f| {
                let name = f.ident.as_ref().unwrap();

                quote! { (self.#name == other.#name) }
            });

            quote! {
                impl #impl_generics ::core::cmp::PartialEq for #name #ty_generics #where_clause {
                    fn eq(&self, other: &Self) -> bool {
                        #(#fs)&&*
                    }
                }
                impl #impl_generics ::core::cmp::Eq for #name #ty_generics #where_clause {
                    fn assert_receiver_is_total_eq(&self) {}
                }
            }
            .into()
        }
        syn::Data::Enum(syn::DataEnum { .. }) => quote! {
            impl #impl_generics ::core::cmp::PartialEq for #name #ty_generics #where_clause {
                #[magic(adt::enum_eq)]
                fn eq(&self, other: &Self) -> bool {
                    crate::prelude::compiler_magic!()
                }
                #[allow(clippy::partialeq_ne_impl)]
                #[magic(adt::enum_ne)]
                fn ne(&self, other: &Self) -> bool {
                    crate::prelude::compiler_magic!()
                }
            }
            impl #impl_generics ::core::cmp::Eq for #name #ty_generics #where_clause {
                fn assert_receiver_is_total_eq(&self) {}
            }
        }
        .into(),
        _ => todo!("HEq macro is not implemented for union type"),
    }
}

#[proc_macro_derive(Interface)]
pub fn interface(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let vis = &ast.vis;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let name = &ast.ident;
    let fname = format!("{name}Fwd");
    let fident = syn::Ident::new(&fname, name.span());
    let bname = format!("{name}Bwd");
    let bident = syn::Ident::new(&bname, name.span());
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        todo!()
    };

    // fields for forward value.
    let fwd_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #vis #name: <#ty as Interface>::Fwd }
    });

    // fields for backward value.
    let bwd_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #vis #name: <#ty as Interface>::Bwd }
    });

    let expanded = quote! {
        #[allow(unused_braces, missing_docs)]
        #[derive(Debug, Clone, Copy)]
        #vis struct #fident #impl_generics #where_clause {
            #(#fwd_fields,)*
        }
        #[allow(unused_braces, missing_docs)]
        #[derive(Debug, Clone, Copy)]
        #vis struct #bident #impl_generics #where_clause {
            #(#bwd_fields,)*
        }

        #[allow(unused_braces, missing_docs)]
        #[::hazardflow_macro::magic(interface::composite_interface)]
        impl #impl_generics Interface for #name #ty_generics #where_clause {
            type Fwd = #fident #ty_generics;
            type Bwd = #bident #ty_generics;
        }
    };
    expanded.into()
}
