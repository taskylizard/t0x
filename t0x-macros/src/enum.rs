//! Code generation for enum types

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Result, Variant};

use crate::attr::{ContainerAttr, FieldAttr};

pub fn generate(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    container_attr: &ContainerAttr,
) -> Result<TokenStream> {
    if container_attr.untagged {
        generate_untagged(variants, container_attr)
    } else if let Some(ref tag) = container_attr.tag {
        if let Some(ref content) = container_attr.content {
            generate_adjacently_tagged(variants, container_attr, tag, content)
        } else {
            generate_internally_tagged(variants, container_attr, tag)
        }
    } else {
        generate_externally_tagged(variants, container_attr)
    }
}

fn generate_untagged(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    container_attr: &ContainerAttr,
) -> Result<TokenStream> {
    let mut variant_types = Vec::new();

    for variant in variants {
        let field_attr = FieldAttr::from_attrs(&variant.attrs)?;
        if field_attr.skip {
            continue;
        }

        let variant_type = generate_variant_type(&variant.fields, container_attr)?;
        variant_types.push(variant_type);
    }

    Ok(quote! {
        let allocator = ast.allocator;
        let mut types = ::t0x::__private::OxcVec::new_in(allocator);

        #(
            types.push(#variant_types);
        )*

        ::t0x::__private::TSType::TSUnionType(ast.alloc_ts_union_type(::t0x::__private::SPAN, types))
    })
}

fn generate_externally_tagged(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    container_attr: &ContainerAttr,
) -> Result<TokenStream> {
    let mut variant_builders = Vec::new();

    for variant in variants {
        let field_attr = FieldAttr::from_attrs(&variant.attrs)?;
        if field_attr.skip {
            continue;
        }

        let variant_name = &variant.ident;
        let ts_variant_name = field_attr
            .rename
            .clone()
            .unwrap_or_else(|| variant_name.to_string());

        let variant_type = match &variant.fields {
            Fields::Unit => {
                quote! {
                    {
                        let key = ::t0x::__private::PropertyKey::StaticIdentifier(
                            ast.alloc_identifier_name(::t0x::__private::SPAN, #ts_variant_name)
                        );
                        let undefined_type = ::t0x::__private::TSType::TSUndefinedKeyword(
                            ast.alloc_ts_undefined_keyword(::t0x::__private::SPAN)
                        );
                        let type_annotation = ast.ts_type_annotation(::t0x::__private::SPAN, undefined_type);

                        let mut members = ::t0x::__private::OxcVec::with_capacity_in(1, allocator);
                        members.push(::t0x::__private::TSSignature::TSPropertySignature(
                            ast.alloc_ts_property_signature(
                                ::t0x::__private::SPAN,
                                false,
                                false,
                                false,
                                key,
                                Some(ast.alloc(type_annotation)),
                            )
                        ));

                        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
                    }
                }
            }
            Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                let inner_ty = &unnamed.unnamed.first().unwrap().ty;
                quote! {
                    {
                        let key = ::t0x::__private::PropertyKey::StaticIdentifier(
                            ast.alloc_identifier_name(::t0x::__private::SPAN, #ts_variant_name)
                        );
                        let inner_type = <#inner_ty as ::t0x::T0x>::ts_type(ast);
                        let type_annotation = ast.ts_type_annotation(::t0x::__private::SPAN, inner_type);

                        let mut members = ::t0x::__private::OxcVec::with_capacity_in(1, allocator);
                        members.push(::t0x::__private::TSSignature::TSPropertySignature(
                            ast.alloc_ts_property_signature(
                                ::t0x::__private::SPAN,
                                false,
                                false,
                                false,
                                key,
                                Some(ast.alloc(type_annotation)),
                            )
                        ));

                        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let field_types: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
                quote! {
                    {
                        let key = ::t0x::__private::PropertyKey::StaticIdentifier(
                            ast.alloc_identifier_name(::t0x::__private::SPAN, #ts_variant_name)
                        );

                        let mut elements = ::t0x::__private::OxcVec::new_in(allocator);
                        #(
                            elements.push(::t0x::__private::TSTupleElement::from(<#field_types as ::t0x::T0x>::ts_type(ast)));
                        )*
                        let tuple_type = ::t0x::__private::TSType::TSTupleType(
                            ast.alloc_ts_tuple_type(::t0x::__private::SPAN, elements)
                        );
                        let type_annotation = ast.ts_type_annotation(::t0x::__private::SPAN, tuple_type);

                        let mut members = ::t0x::__private::OxcVec::with_capacity_in(1, allocator);
                        members.push(::t0x::__private::TSSignature::TSPropertySignature(
                            ast.alloc_ts_property_signature(
                                ::t0x::__private::SPAN,
                                false,
                                false,
                                false,
                                key,
                                Some(ast.alloc(type_annotation)),
                            )
                        ));

                        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
                    }
                }
            }
            Fields::Named(named) => {
                let inner_type = generate_named_fields_type(&named.named, container_attr)?;
                quote! {
                    {
                        let key = ::t0x::__private::PropertyKey::StaticIdentifier(
                            ast.alloc_identifier_name(::t0x::__private::SPAN, #ts_variant_name)
                        );
                        let inner_type = #inner_type;
                        let type_annotation = ast.ts_type_annotation(::t0x::__private::SPAN, inner_type);

                        let mut members = ::t0x::__private::OxcVec::with_capacity_in(1, allocator);
                        members.push(::t0x::__private::TSSignature::TSPropertySignature(
                            ast.alloc_ts_property_signature(
                                ::t0x::__private::SPAN,
                                false,
                                false,
                                false,
                                key,
                                Some(ast.alloc(type_annotation)),
                            )
                        ));

                        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
                    }
                }
            }
        };

        variant_builders.push(variant_type);
    }

    Ok(quote! {
        let allocator = ast.allocator;
        let mut types = ::t0x::__private::OxcVec::new_in(allocator);

        #(
            types.push(#variant_builders);
        )*

        ::t0x::__private::TSType::TSUnionType(ast.alloc_ts_union_type(::t0x::__private::SPAN, types))
    })
}

fn generate_internally_tagged(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    container_attr: &ContainerAttr,
    tag: &str,
) -> Result<TokenStream> {
    let mut variant_builders = Vec::new();

    for variant in variants {
        let field_attr = FieldAttr::from_attrs(&variant.attrs)?;
        if field_attr.skip {
            continue;
        }

        let variant_name = &variant.ident;
        let ts_variant_name = field_attr
            .rename
            .clone()
            .unwrap_or_else(|| variant_name.to_string());

        let variant_type = match &variant.fields {
            Fields::Unit => {
                quote! {
                    {
                        let mut members = ::t0x::__private::OxcVec::with_capacity_in(1, allocator);

                        let tag_key = ::t0x::__private::PropertyKey::StaticIdentifier(
                            ast.alloc_identifier_name(::t0x::__private::SPAN, #tag)
                        );
                        let tag_lit = ast.ts_literal_type(
                            ::t0x::__private::SPAN,
                            ::t0x::__private::TSLiteral::StringLiteral(
                                ast.alloc_string_literal(::t0x::__private::SPAN, #ts_variant_name, None)
                            )
                        );
                        let tag_annotation = ast.ts_type_annotation(
                            ::t0x::__private::SPAN,
                            ::t0x::__private::TSType::TSLiteralType(ast.alloc(tag_lit))
                        );
                        members.push(::t0x::__private::TSSignature::TSPropertySignature(
                            ast.alloc_ts_property_signature(
                                ::t0x::__private::SPAN,
                                false,
                                false,
                                false,
                                tag_key,
                                Some(ast.alloc(tag_annotation)),
                            )
                        ));

                        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
                    }
                }
            }
            Fields::Named(named) => {
                let inner_fields = generate_named_fields_members(&named.named, container_attr)?;
                quote! {
                    {
                        let mut members = ::t0x::__private::OxcVec::new_in(allocator);

                        let tag_key = ::t0x::__private::PropertyKey::StaticIdentifier(
                            ast.alloc_identifier_name(::t0x::__private::SPAN, #tag)
                        );
                        let tag_lit = ast.ts_literal_type(
                            ::t0x::__private::SPAN,
                            ::t0x::__private::TSLiteral::StringLiteral(
                                ast.alloc_string_literal(::t0x::__private::SPAN, #ts_variant_name, None)
                            )
                        );
                        let tag_annotation = ast.ts_type_annotation(
                            ::t0x::__private::SPAN,
                            ::t0x::__private::TSType::TSLiteralType(ast.alloc(tag_lit))
                        );
                        members.push(::t0x::__private::TSSignature::TSPropertySignature(
                            ast.alloc_ts_property_signature(
                                ::t0x::__private::SPAN,
                                false,
                                false,
                                false,
                                tag_key,
                                Some(ast.alloc(tag_annotation)),
                            )
                        ));

                        #inner_fields

                        ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
                    }
                }
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Internally tagged enums cannot have tuple variants",
                ));
            }
        };

        variant_builders.push(variant_type);
    }

    Ok(quote! {
        let allocator = ast.allocator;
        let mut types = ::t0x::__private::OxcVec::new_in(allocator);

        #(
            types.push(#variant_builders);
        )*

        ::t0x::__private::TSType::TSUnionType(ast.alloc_ts_union_type(::t0x::__private::SPAN, types))
    })
}

fn generate_adjacently_tagged(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    container_attr: &ContainerAttr,
    tag: &str,
    content: &str,
) -> Result<TokenStream> {
    let mut variant_builders = Vec::new();

    for variant in variants {
        let field_attr = FieldAttr::from_attrs(&variant.attrs)?;
        if field_attr.skip {
            continue;
        }

        let variant_name = &variant.ident;
        let ts_variant_name = field_attr
            .rename
            .clone()
            .unwrap_or_else(|| variant_name.to_string());

        let content_type = generate_variant_type(&variant.fields, container_attr)?;

        variant_builders.push(quote! {
            {
                let mut members = ::t0x::__private::OxcVec::with_capacity_in(2, allocator);

                let tag_key = ::t0x::__private::PropertyKey::StaticIdentifier(
                    ast.alloc_identifier_name(::t0x::__private::SPAN, #tag)
                );
                let tag_lit = ast.ts_literal_type(
                    ::t0x::__private::SPAN,
                    ::t0x::__private::TSLiteral::StringLiteral(
                        ast.alloc_string_literal(::t0x::__private::SPAN, #ts_variant_name, None)
                    )
                );
                let tag_annotation = ast.ts_type_annotation(
                    ::t0x::__private::SPAN,
                    ::t0x::__private::TSType::TSLiteralType(ast.alloc(tag_lit))
                );
                members.push(::t0x::__private::TSSignature::TSPropertySignature(
                    ast.alloc_ts_property_signature(
                        ::t0x::__private::SPAN,
                        false,
                        false,
                        false,
                        tag_key,
                        Some(ast.alloc(tag_annotation)),
                    )
                ));

                let content_key = ::t0x::__private::PropertyKey::StaticIdentifier(
                    ast.alloc_identifier_name(::t0x::__private::SPAN, #content)
                );
                let content_type = #content_type;
                let content_annotation = ast.ts_type_annotation(::t0x::__private::SPAN, content_type);
                members.push(::t0x::__private::TSSignature::TSPropertySignature(
                    ast.alloc_ts_property_signature(
                        ::t0x::__private::SPAN,
                        false,
                        false,
                        false,
                        content_key,
                        Some(ast.alloc(content_annotation)),
                    )
                ));

                ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
            }
        });
    }

    Ok(quote! {
        let allocator = ast.allocator;
        let mut types = ::t0x::__private::OxcVec::new_in(allocator);

        #(
            types.push(#variant_builders);
        )*

        ::t0x::__private::TSType::TSUnionType(ast.alloc_ts_union_type(::t0x::__private::SPAN, types))
    })
}

fn generate_variant_type(fields: &Fields, container_attr: &ContainerAttr) -> Result<TokenStream> {
    match fields {
        Fields::Unit => Ok(quote! {
            ::t0x::__private::TSType::TSUndefinedKeyword(ast.alloc_ts_undefined_keyword(::t0x::__private::SPAN))
        }),
        Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
            let ty = &unnamed.unnamed.first().unwrap().ty;
            Ok(quote! { <#ty as ::t0x::T0x>::ts_type(ast) })
        }
        Fields::Unnamed(unnamed) => {
            let field_types: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
            Ok(quote! {
                {
                    let mut elements = ::t0x::__private::OxcVec::new_in(allocator);
                    #(
                        elements.push(::t0x::__private::TSTupleElement::from(<#field_types as ::t0x::T0x>::ts_type(ast)));
                    )*
                    ::t0x::__private::TSType::TSTupleType(ast.alloc_ts_tuple_type(::t0x::__private::SPAN, elements))
                }
            })
        }
        Fields::Named(named) => generate_named_fields_type(&named.named, container_attr),
    }
}

fn generate_named_fields_type(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    container_attr: &ContainerAttr,
) -> Result<TokenStream> {
    let members = generate_named_fields_members(fields, container_attr)?;
    Ok(quote! {
        {
            let mut members = ::t0x::__private::OxcVec::new_in(allocator);
            #members
            ::t0x::__private::TSType::TSTypeLiteral(ast.alloc_ts_type_literal(::t0x::__private::SPAN, members))
        }
    })
}

fn generate_named_fields_members(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    container_attr: &ContainerAttr,
) -> Result<TokenStream> {
    let mut field_builders = Vec::new();

    for field in fields {
        let field_attr = FieldAttr::from_attrs(&field.attrs)?;
        if field_attr.skip {
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
        let field_docs: Vec<String> = field_attr.docs.clone();

        field_builders.push(quote! {
            {
                let docs: &[&str] = &[#(#field_docs),*];
                ::t0x::__private::register_field_doc(#ts_field_name, docs);

                let key = ::t0x::__private::PropertyKey::StaticIdentifier(
                    ast.alloc_identifier_name(::t0x::__private::SPAN, #ts_field_name)
                );
                let field_type = <#ty as ::t0x::T0x>::ts_type(ast);
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

    Ok(quote! { #(#field_builders)* })
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}
