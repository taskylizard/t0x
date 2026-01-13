//! T0x implementations for serde_json types

use crate::T0x;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{AstBuilder, NONE, ast::TSType};
use oxc_span::SPAN;

/// Returns the TypeScript type definition for `JsonValue`.
///
/// This should be included in the generated output when using `serde_json::Value`
/// in your types, as the generated types will reference `JsonValue`.
///
/// # Example
///
/// ```rust
/// use t0x::serde_json::json_value_def;
///
/// let mut output = String::new();
/// output.push_str(&json_value_def());
/// // output.push_str(&export!(MyType, ...));
/// ```
pub fn json_value_def() -> String {
    <serde_json::Value as T0x>::type_def()
}

impl T0x for serde_json::Value {
    const NAME: &'static str = "JsonValue";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        use oxc_ast::ast::TSMappedTypeModifierOperator;

        let allocator = ast.allocator;
        let mut types = OxcVec::with_capacity_in(6, allocator);

        // number
        types.push(TSType::TSNumberKeyword(ast.alloc_ts_number_keyword(SPAN)));
        // string
        types.push(TSType::TSStringKeyword(ast.alloc_ts_string_keyword(SPAN)));
        // boolean
        types.push(TSType::TSBooleanKeyword(ast.alloc_ts_boolean_keyword(SPAN)));

        // Array<JsonValue>
        let array_name = ast.ts_type_name_identifier_reference(SPAN, "Array");

        let self_ref = ast.ts_type_name_identifier_reference(SPAN, "JsonValue");
        let mut array_args = OxcVec::with_capacity_in(1, allocator);
        array_args.push(TSType::TSTypeReference(
            ast.alloc_ts_type_reference(SPAN, self_ref, NONE),
        ));

        let array_params = ast.ts_type_parameter_instantiation(SPAN, array_args);
        types.push(TSType::TSTypeReference(ast.alloc_ts_type_reference(
            SPAN,
            array_name,
            Some(array_params),
        )));

        // { [key in string]?: JsonValue }
        let type_param = ast.ts_type_parameter(
            SPAN,
            ast.binding_identifier(SPAN, "key"),
            Some(TSType::TSStringKeyword(ast.alloc_ts_string_keyword(SPAN))),
            None,
            true,  // in
            false, // out
            false, // const
        );
        let value_ref = ast.ts_type_name_identifier_reference(SPAN, "JsonValue");
        let value_type =
            TSType::TSTypeReference(ast.alloc_ts_type_reference(SPAN, value_ref, NONE));
        let mapped_type = ast.ts_mapped_type(
            SPAN,
            type_param,
            None,
            Some(value_type),
            Some(TSMappedTypeModifierOperator::True),
            None,
        );
        types.push(TSType::TSMappedType(ast.alloc(mapped_type)));

        // null
        types.push(TSType::TSNullKeyword(ast.alloc_ts_null_keyword(SPAN)));

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
