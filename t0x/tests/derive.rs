//! Tests for derive macro

#![allow(dead_code)]

use t0x::T0x;

#[test]
fn test_simple_struct() {
    #[derive(T0x)]
    struct SimpleStruct {
        name: String,
        count: u32,
    }

    let output = SimpleStruct::type_def();

    assert!(output.contains("type SimpleStruct"));
    assert!(output.contains("name: string"));
    assert!(output.contains("count: number"));
}

#[test]
fn test_struct_with_docs() {
    /// A documented struct
    #[derive(T0x)]
    struct DocStruct {
        value: i32,
    }

    let output = DocStruct::type_def();

    assert!(output.contains("A documented struct"));
    assert!(output.contains("type DocStruct"));
}

#[test]
fn test_struct_rename() {
    #[derive(T0x)]
    #[t0x(rename = "RenamedType")]
    struct OriginalName {
        field: bool,
    }

    let output = OriginalName::type_def();

    assert!(output.contains("type RenamedType"));
}

#[test]
fn test_field_rename() {
    #[derive(T0x)]
    struct FieldRename {
        #[t0x(rename = "renamedField")]
        original_field: String,
    }

    let output = FieldRename::type_def();

    assert!(output.contains("renamedField: string"));
}

#[test]
fn test_rename_all_camel_case() {
    #[derive(T0x)]
    #[t0x(rename_all = "camelCase")]
    struct CamelCaseStruct {
        first_name: String,
        last_name: String,
    }

    let output = CamelCaseStruct::type_def();

    assert!(output.contains("firstName: string"));
    assert!(output.contains("lastName: string"));
}

#[test]
fn test_optional_field() {
    #[derive(T0x)]
    struct WithOptional {
        required: String,
        optional: Option<u32>,
    }

    let output = WithOptional::type_def();

    assert!(output.contains("required: string"));
    assert!(output.contains("optional?:"));
}

#[test]
fn test_skip_field() {
    #[derive(T0x)]
    struct WithSkipped {
        included: String,
        #[t0x(skip)]
        skipped: i32,
    }

    let output = WithSkipped::type_def();

    assert!(output.contains("included: string"));
    assert!(!output.contains("skipped"));
}

#[test]
fn test_externally_tagged_enum() {
    #[derive(T0x)]
    enum ExternallyTagged {
        Variant1,
        Variant2(String),
        Variant3 { name: String },
    }

    let output = ExternallyTagged::type_def();

    assert!(output.contains("Variant1"));
    assert!(output.contains("Variant2"));
    assert!(output.contains("Variant3"));
}

#[test]
fn test_internally_tagged_enum() {
    #[derive(T0x)]
    #[t0x(tag = "type")]
    enum InternallyTagged {
        Unit,
        Named { value: i32 },
    }

    let output = InternallyTagged::type_def();

    assert!(output.contains(r#"type: "Unit""#));
    assert!(output.contains(r#"type: "Named""#));
    assert!(output.contains("value: number"));
}

#[test]
fn test_adjacently_tagged_enum() {
    #[derive(T0x)]
    #[t0x(tag = "t", content = "c")]
    enum AdjacentlyTagged {
        First(u32),
        Second { x: i32 },
    }

    let output = AdjacentlyTagged::type_def();

    assert!(output.contains(r#"t: "First""#));
    assert!(output.contains(r#"t: "Second""#));
    assert!(output.contains("c:"));
}

#[test]
fn test_untagged_enum() {
    #[derive(T0x)]
    #[t0x(untagged)]
    enum Untagged {
        Text(String),
        Number(i32),
    }

    let output = Untagged::type_def();

    assert!(output.contains("string | number"));
}

#[test]
fn test_tuple_struct() {
    #[derive(T0x)]
    struct TupleStruct(String, u32);

    let output = TupleStruct::type_def();

    assert!(output.contains("[string, number]"));
}

#[test]
fn test_newtype_struct() {
    #[derive(T0x)]
    struct Newtype(String);

    let output = Newtype::type_def();

    assert!(output.contains("string"));
}

#[test]
fn test_field_docs() {
    #[derive(T0x)]
    struct WithFieldDocs {
        /// The user's name
        name: String,
        /// The user's age in years
        age: u32,
    }

    let output = WithFieldDocs::type_def();

    assert!(output.contains("/** The user's name */"));
    assert!(output.contains("/** The user's age in years */"));
}

#[test]
fn test_bigint_field() {
    #[derive(T0x)]
    struct WithBigInt {
        small: u32,
        big: u64,
    }

    let output = WithBigInt::type_def();

    assert!(output.contains("small: number"));
    assert!(output.contains("big: bigint"));
}
