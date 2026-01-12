//! Tests for primitive type conversions

use t0x::T0x;

#[test]
fn test_number_types() {
    let u8_output = u8::type_def();
    assert!(u8_output.contains("number"));

    let i32_output = i32::type_def();
    assert!(i32_output.contains("number"));

    let f64_output = f64::type_def();
    assert!(f64_output.contains("number"));
}

#[test]
fn test_bigint_types() {
    let u64_output = u64::type_def();
    assert!(u64_output.contains("bigint"));

    let i128_output = i128::type_def();
    assert!(i128_output.contains("bigint"));
}

#[test]
fn test_string_types() {
    let string_output = String::type_def();
    assert!(string_output.contains("string"));
}

#[test]
fn test_bool_type() {
    let bool_output = bool::type_def();
    assert!(bool_output.contains("boolean"));
}

#[test]
fn test_option_type() {
    let output = <Option<String>>::type_def();
    assert!(output.contains("string"));
}

#[test]
fn test_vec_type() {
    let output = <Vec<u32>>::type_def();
    assert!(output.contains("number[]"));
}
