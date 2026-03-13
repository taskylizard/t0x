//! t0x - Generate TypeScript types from Rust types using oxc
//!
//! This library provides the [`T0x`] trait which allows Rust types to be converted
//! to TypeScript type declarations using oxc's AST builder.
//!
//! ## Example
//!
//! ```rust
//! use t0x::T0x;
//!
//! #[derive(T0x)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! let ts_code = User::type_def();
//! // Results in: type User = { id: number; name: string; };
//! ```
//!
//! ## Exporting Multiple Types
//!
//! Use [`export`] to generate multiple types at once:
//!
//! ```rust
//! use t0x::{T0x, export};
//!
//! #[derive(T0x)]
//! struct User { id: u32 }
//!
//! #[derive(T0x)]
//! struct Post { title: String }
//!
//! let output = export!(User, Post);
//! ```

mod impls;
mod output;
#[cfg(feature = "serde-json-impl")]
pub mod serde_json;

pub use t0x_macros::T0x;

use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::TSType};

use output::{TypeOutput, generate_ts_code};

pub trait T0x {
    const NAME: &'static str;

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a>;

    fn ts_docs() -> Option<&'static str> {
        None
    }

    fn value_def() -> Option<String> {
        None
    }

    fn options_def() -> Option<String> {
        None
    }

    fn type_def() -> String {
        let allocator = Allocator::default();
        let output = TypeOutput {
            name: Self::NAME,
            docs: Self::ts_docs(),
            build_type: |ast| Self::ts_type(ast),
        };
        generate_ts_code(&allocator, &output)
    }
}

/// Generates TypeScript type definitions for multiple types.
///
/// # Example
///
/// ```rust
/// use t0x::{T0x, export};
///
/// #[derive(T0x)]
/// struct User { id: u32 }
///
/// #[derive(T0x)]
/// struct Post { title: String }
///
/// let output = export!(User, Post);
/// // Contains both type definitions
/// ```
#[macro_export]
macro_rules! export {
    ($($ty:ty),* $(,)?) => {{
        let mut output = String::new();
        $(
            output.push_str(&<$ty as $crate::T0x>::type_def());
            output.push('\n');
            if let Some(options_def) = <$ty as $crate::T0x>::options_def() {
                output.push_str(&options_def);
                output.push('\n');
            }
            if let Some(value_def) = <$ty as $crate::T0x>::value_def() {
                output.push_str(&value_def);
                output.push('\n');
            }
        )*
        output
    }};
}

#[doc(hidden)]
pub mod __private {
    pub use oxc_allocator::{Allocator, Vec as OxcVec};
    pub use oxc_ast::{AstBuilder, NONE, ast::*};
    pub use oxc_span::{Atom, SPAN};

    pub use crate::output::{TypeOutput, generate_ts_code};

    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        pub static FIELD_DOCS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    }

    pub fn register_field_doc(field_name: &str, docs: &[&str]) {
        if docs.is_empty() {
            return;
        }
        let doc_str = docs.join(" ").trim().to_string();
        if doc_str.is_empty() {
            return;
        }
        FIELD_DOCS.with(|fd| {
            fd.borrow_mut()
                .insert(field_name.to_string(), format!("/** {} */", doc_str));
        });
    }

    pub fn take_field_docs() -> HashMap<String, String> {
        FIELD_DOCS.with(|fd| std::mem::take(&mut *fd.borrow_mut()))
    }

    pub fn format_docs(docs: &[&str]) -> String {
        match docs {
            [] => String::new(),
            [doc] if doc.contains('\n') => format!("/**{doc}*/\n"),
            _ => {
                let mut buffer = String::from("/**\n");
                let mut lines = docs.iter().peekable();

                while let Some(line) = lines.next() {
                    buffer.push_str(" *");
                    buffer.push_str(line);

                    if lines.peek().is_some() {
                        buffer.push('\n');
                    }
                }
                buffer.push_str("\n */\n");
                buffer
            }
        }
    }
}
