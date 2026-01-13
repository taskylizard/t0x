//! T0x implementations for serde_json types

use crate::T0x;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{AstBuilder, NONE, ast::TSType};
use oxc_span::SPAN;

impl T0x for serde_json::Value {
    const NAME: &'static str = "JsonValue";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let allocator = ast.allocator;
        let mut types = OxcVec::with_capacity_in(6, allocator);

        types.push(TSType::TSNumberKeyword(ast.alloc_ts_number_keyword(SPAN)));
        types.push(TSType::TSStringKeyword(ast.alloc_ts_string_keyword(SPAN)));
        types.push(TSType::TSBooleanKeyword(ast.alloc_ts_boolean_keyword(SPAN)));
        types.push(TSType::TSNullKeyword(ast.alloc_ts_null_keyword(SPAN)));

        let self_ref = ast.ts_type_name_identifier_reference(SPAN, "JsonValue");
        let array_element =
            TSType::TSTypeReference(ast.alloc_ts_type_reference(SPAN, self_ref, NONE));
        types.push(TSType::TSArrayType(
            ast.alloc_ts_array_type(SPAN, array_element),
        ));

        let record_name = ast.ts_type_name_identifier_reference(SPAN, "Record");
        let mut record_args = OxcVec::with_capacity_in(2, allocator);
        record_args.push(TSType::TSStringKeyword(ast.alloc_ts_string_keyword(SPAN)));
        let value_ref = ast.ts_type_name_identifier_reference(SPAN, "JsonValue");
        record_args.push(TSType::TSTypeReference(
            ast.alloc_ts_type_reference(SPAN, value_ref, NONE),
        ));
        let record_params = ast.ts_type_parameter_instantiation(SPAN, record_args);
        types.push(TSType::TSTypeReference(ast.alloc_ts_type_reference(
            SPAN,
            record_name,
            Some(record_params),
        )));

        TSType::TSUnionType(ast.alloc_ts_union_type(SPAN, types))
    }
}

impl T0x for serde_json::Number {
    const NAME: &'static str = "number";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        TSType::TSNumberKeyword(ast.alloc_ts_number_keyword(SPAN))
    }
}

impl<K: T0x, V: T0x> T0x for serde_json::Map<K, V> {
    const NAME: &'static str = "Map";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        let allocator = ast.allocator;

        let type_name = ast.ts_type_name_identifier_reference(SPAN, "Record");

        let mut type_args = OxcVec::with_capacity_in(2, allocator);
        type_args.push(K::ts_type(ast));
        type_args.push(V::ts_type(ast));

        let type_params = ast.ts_type_parameter_instantiation(SPAN, type_args);

        TSType::TSTypeReference(ast.alloc_ts_type_reference(SPAN, type_name, Some(type_params)))
    }
}
