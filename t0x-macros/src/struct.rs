//! Code generation for struct types

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Result};

use crate::attr::{ContainerAttr, FieldAttr};

pub fn generate(
    fields: &Fields,
    container_attr: &ContainerAttr,
    skip_excluded: bool,
) -> Result<TokenStream> {
    match fields {
        Fields::Named(named) => generate_named_struct(&named.named, container_attr, skip_excluded),
        Fields::Unnamed(unnamed) => generate_tuple_struct(&unnamed.unnamed, container_attr),
        Fields::Unit => generate_unit_struct(),
    }
}

fn generate_named_struct(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    container_attr: &ContainerAttr,
    skip_excluded: bool,
) -> Result<TokenStream> {
    let mut field_builders = Vec::new();

    for field in fields {
        let field_attr = FieldAttr::from_attrs(&field.attrs)?;

        if field_attr.skip {
            continue;
        }

        if skip_excluded && field_attr.exclude {
            continue;
        }

        let field_name = field.ident.as_ref().unwrap();
        let ts_field_name = field_attr.rename.clone().unwrap_or_else(|| {
            let name = field_name.to_string();
            if let Some(ref rule) = container_attr.rename_all {
                rule.apply(&name)
            } else {
                name
            }
        });

        let ty = &field.ty;
        let is_optional = field_attr.optional || is_option_type(ty);

        let type_expr = if let Some(ref type_override) = field_attr.r#type {
            quote! {
                {
                    let type_name = ast.ts_type_name_identifier_reference(
                        ::t0x::__private::SPAN,
                        #type_override
                    );
                    ::t0x::__private::TSType::TSTypeReference(
                        ast.alloc_ts_type_reference(::t0x::__private::SPAN, type_name, ::t0x::__private::NONE)
                    )
                }
            }
        } else {
            quote! { <#ty as ::t0x::T0x>::ts_type(ast) }
        };

        let field_docs: Vec<String> = field_attr.docs.clone();

        if field_attr.flatten {
            field_builders.push(quote! {
                {
                    let flattened_type = #type_expr;
                    if let ::t0x::__private::TSType::TSTypeLiteral(lit) = flattened_type {
                        for member in lit.unbox().members {
                            members.push(member);
                        }
                    }
                }
            });
        } else {
            field_builders.push(quote! {
                {
                    let docs: &[&str] = &[#(#field_docs),*];
                    ::t0x::__private::register_field_doc(#ts_field_name, docs);

                    let key = ::t0x::__private::PropertyKey::StaticIdentifier(
                        ast.alloc_identifier_name(::t0x::__private::SPAN, #ts_field_name)
                    );
                    let field_type = #type_expr;
                    let type_annotation = ast.ts_type_annotation(::t0x::__private::SPAN, field_type);

                    members.push(::t0x::__private::TSSignature::TSPropertySignature(
                        ast.alloc_ts_property_signature(
                            ::t0x::__private::SPAN,
                            false,
                            #is_optional,
                            false,
                            key,
                            Some(ast.alloc(type_annotation)),
                        )
                    ));
                }
            });
        }
    }

    Ok(quote! {
        let allocator = ast.allocator;
        let mut members = ::t0x::__private::OxcVec::new_in(allocator);

        #(#field_builders)*

        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
    })
}

fn generate_tuple_struct(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    _container_attr: &ContainerAttr,
) -> Result<TokenStream> {
    if fields.len() == 1 {
        let ty = &fields.first().unwrap().ty;
        return Ok(quote! {
            <#ty as ::t0x::T0x>::ts_type(ast)
        });
    }

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    Ok(quote! {
        let allocator = ast.allocator;
        let mut elements = ::t0x::__private::OxcVec::new_in(allocator);

        #(
            elements.push(::t0x::__private::TSTupleElement::from(<#field_types as ::t0x::T0x>::ts_type(ast)));
        )*

        ::t0x::__private::TSType::TSTupleType(ast.alloc_ts_tuple_type(::t0x::__private::SPAN, elements))
    })
}

fn generate_unit_struct() -> Result<TokenStream> {
    Ok(quote! {
        ::t0x::__private::TSType::TSUndefinedKeyword(ast.alloc_ts_undefined_keyword(::t0x::__private::SPAN))
    })
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}
