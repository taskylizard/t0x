//! T0x implementations for primitive and standard library types

use crate::T0x;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    AstBuilder,
    ast::{PropertyKey, TSSignature, TSTupleElement, TSType},
};
use oxc_span::SPAN;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

macro_rules! impl_primitive_number {
    ($($ty:ty),* $(,)?) => {
        $(
            impl T0x for $ty {
                const NAME: &'static str = stringify!($ty);

                fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
                    TSType::TSNumberKeyword(ast.alloc_ts_number_keyword(SPAN))
                }
            }
        )*
    };
}

macro_rules! impl_primitive_bigint {
    ($($ty:ty),* $(,)?) => {
        $(
            impl T0x for $ty {
                const NAME: &'static str = stringify!($ty);

                fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
                    TSType::TSBigIntKeyword(ast.alloc_ts_big_int_keyword(SPAN))
                }
            }
        )*
    };
}

macro_rules! impl_primitive_string {
    ($($ty:ty),* $(,)?) => {
        $(
            impl T0x for $ty {
                const NAME: &'static str = stringify!($ty);

                fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
                    TSType::TSStringKeyword(ast.alloc_ts_string_keyword(SPAN))
                }
            }
        )*
    };
}

impl_primitive_number!(u8, u16, u32, i8, i16, i32, f32, f64, usize, isize);
impl_primitive_bigint!(u64, u128, i64, i128);
impl_primitive_string!(String, str, char);

impl T0x for bool {
    const NAME: &'static str = "bool";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        TSType::TSBooleanKeyword(ast.alloc_ts_boolean_keyword(SPAN))
    }
}

impl T0x for () {
    const NAME: &'static str = "()";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        TSType::TSUndefinedKeyword(ast.alloc_ts_undefined_keyword(SPAN))
    }
}

impl<T: T0x> T0x for Option<T> {
    const NAME: &'static str = "Option";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        T::ts_type(ast)
    }
}

impl<T: T0x> T0x for Vec<T> {
    const NAME: &'static str = "Vec";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let element_type = T::ts_type(ast);
        TSType::TSArrayType(ast.alloc_ts_array_type(SPAN, element_type))
    }
}

impl<T: T0x, const N: usize> T0x for [T; N] {
    const NAME: &'static str = "Array";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let element_type = T::ts_type(ast);
        TSType::TSArrayType(ast.alloc_ts_array_type(SPAN, element_type))
    }
}

impl<T: T0x> T0x for [T] {
    const NAME: &'static str = "Slice";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let element_type = T::ts_type(ast);
        TSType::TSArrayType(ast.alloc_ts_array_type(SPAN, element_type))
    }
}

impl<T: T0x> T0x for Box<T> {
    const NAME: &'static str = "Box";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        T::ts_type(ast)
    }
}

impl<T: T0x + ?Sized> T0x for &T {
    const NAME: &'static str = "Ref";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        T::ts_type(ast)
    }
}

impl<T: T0x + ?Sized> T0x for &mut T {
    const NAME: &'static str = "MutRef";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        T::ts_type(ast)
    }
}

impl<K: T0x, V: T0x> T0x for HashMap<K, V> {
    const NAME: &'static str = "HashMap";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        build_record_type(ast, K::ts_type(ast), V::ts_type(ast))
    }
}

impl<K: T0x, V: T0x> T0x for BTreeMap<K, V> {
    const NAME: &'static str = "BTreeMap";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        build_record_type(ast, K::ts_type(ast), V::ts_type(ast))
    }
}

impl<T: T0x> T0x for HashSet<T> {
    const NAME: &'static str = "HashSet";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let element_type = T::ts_type(ast);
        TSType::TSArrayType(ast.alloc_ts_array_type(SPAN, element_type))
    }
}

impl<T: T0x> T0x for BTreeSet<T> {
    const NAME: &'static str = "BTreeSet";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let element_type = T::ts_type(ast);
        TSType::TSArrayType(ast.alloc_ts_array_type(SPAN, element_type))
    }
}

impl<T: T0x, E: T0x> T0x for Result<T, E> {
    const NAME: &'static str = "Result";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let allocator = ast.allocator;

        let ok_members = {
            let mut members = OxcVec::with_capacity_in(2, allocator);
            members.push(build_property_signature(ast, "ok", T::ts_type(ast), false));
            members.push(build_property_signature(
                ast,
                "err",
                TSType::TSUndefinedKeyword(ast.alloc_ts_undefined_keyword(SPAN)),
                false,
            ));
            members
        };

        let err_members = {
            let mut members = OxcVec::with_capacity_in(2, allocator);
            members.push(build_property_signature(
                ast,
                "ok",
                TSType::TSUndefinedKeyword(ast.alloc_ts_undefined_keyword(SPAN)),
                false,
            ));
            members.push(build_property_signature(ast, "err", E::ts_type(ast), false));
            members
        };

        let ok_type = TSType::TSTypeLiteral(ast.alloc_ts_type_literal(SPAN, ok_members));
        let err_type = TSType::TSTypeLiteral(ast.alloc_ts_type_literal(SPAN, err_members));

        let mut types = OxcVec::with_capacity_in(2, allocator);
        types.push(ok_type);
        types.push(err_type);

        TSType::TSUnionType(ast.alloc_ts_union_type(SPAN, types))
    }
}

fn build_record_type<'a>(ast: AstBuilder<'a>, key: TSType<'a>, value: TSType<'a>) -> TSType<'a> {
    let allocator = ast.allocator;

    let type_name = ast.ts_type_name_identifier_reference(SPAN, "Record");

    let mut type_args = OxcVec::with_capacity_in(2, allocator);
    type_args.push(key);
    type_args.push(value);

    let type_params = ast.ts_type_parameter_instantiation(SPAN, type_args);

    TSType::TSTypeReference(ast.alloc_ts_type_reference(SPAN, type_name, Some(type_params)))
}

fn build_property_signature<'a>(
    ast: AstBuilder<'a>,
    name: &'a str,
    ts_type: TSType<'a>,
    optional: bool,
) -> TSSignature<'a> {
    let key = PropertyKey::StaticIdentifier(ast.alloc_identifier_name(SPAN, name));
    let type_annotation = ast.ts_type_annotation(SPAN, ts_type);

    TSSignature::TSPropertySignature(ast.alloc_ts_property_signature(
        SPAN,
        false,
        optional,
        false,
        key,
        Some(ast.alloc(type_annotation)),
    ))
}

macro_rules! impl_tuples {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: T0x $(, $rest: T0x)*> T0x for ($first, $($rest,)*) {
            const NAME: &'static str = "Tuple";

            fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
                let allocator = ast.allocator;
                let mut elements = OxcVec::new_in(allocator);

                elements.push(TSTupleElement::from($first::ts_type(ast)));
                $(
                    elements.push(TSTupleElement::from($rest::ts_type(ast)));
                )*

                TSType::TSTupleType(ast.alloc_ts_tuple_type(SPAN, elements))
            }
        }

        impl_tuples!($($rest),*);
    };
}

impl_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
