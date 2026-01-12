//! Derive macros for t0x
//!
//! This crate provides the `#[derive(T0x)]` macro for automatically implementing
//! the `T0x` trait for structs and enums.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, Lit, Meta, Result, parse_macro_input};

mod attr;
mod r#enum;
mod r#struct;

use attr::ContainerAttr;

#[proc_macro_derive(T0x, attributes(t0x))]
pub fn derive_t0x(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_t0x_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_t0x_impl(input: DeriveInput) -> Result<TokenStream2> {
    let container_attr = ContainerAttr::from_attrs(&input.attrs)?;
    let name = &input.ident;
    let ts_name = container_attr
        .rename
        .clone()
        .unwrap_or_else(|| name.to_string());

    let docs = extract_docs(&input.attrs);

    let type_builder = match &input.data {
        Data::Struct(data) => r#struct::generate(&data.fields, &container_attr)?,
        Data::Enum(data) => r#enum::generate(&data.variants, &container_attr)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "T0x cannot be derived for unions",
            ));
        }
    };

    let docs_fn = if docs.is_empty() {
        quote! {}
    } else {
        let docs_str = docs.join("\n");
        quote! {
            fn ts_docs() -> Option<&'static str> {
                Some(#docs_str)
            }
        }
    };

    Ok(quote! {
        impl ::t0x::T0x for #name {
            const NAME: &'static str = #ts_name;

            fn ts_type<'a>(ast: ::t0x::__private::AstBuilder<'a>) -> ::t0x::__private::TSType<'a> {
                #type_builder
            }

            #docs_fn
        }
    })
}

fn extract_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            match &attr.meta {
                Meta::NameValue(nv) => {
                    if let Expr::Lit(expr_lit) = &nv.value
                        && let Lit::Str(lit_str) = &expr_lit.lit
                    {
                        return Some(lit_str.value());
                    }
                    None
                }
                _ => None,
            }
        })
        .collect()
}
