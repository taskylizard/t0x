# 🏗️ t0x (/tiː ˈzɪroʊ ɛks/)

Generate TypeScript types from Rust types using [oxc](https://oxc.rs/).

Unlike string-based approaches, t0x builds proper TypeScript AST nodes using oxc's `AstBuilder`, ensuring syntactically correct output.

## ⚓ Features

- Derive macro: `#[derive(T0x)]` for structs and enums.
- oxc-based generation: Uses oxc's AST builder for proper TypeScript output.
- Type conversion: Rust primitives to TypeScript (with BigInt support for u64/i64/u128/i128)
- Rustdoc comments: Converts `///` doc comments to JSDoc comments (container and field level)
- Serde-like attributes: `rename`, `rename_all`, `skip`, `tag`, `content`, `untagged`

## 📦 Usage

```rust
use t0x::T0x;

/// A user in the system
#[derive(T0x)]
struct User {
    /// The user's unique identifier
    id: u64,
    /// The user's display name
    name: String,
    /// The user's email (optional)
    email: Option<String>,
}

fn main() {
    println!("{}", User::type_def());
}
```

Output:

```typescript
/** A user in the system */
type User = {
  /** The user's unique identifier */
  id: bigint
  /** The user's display name */
  name: string
  /** The user's email (optional) */
  email?: string
}
```

## 🧩 Type Mappings

| Rust Type                                            | TypeScript Type                                          |
| ---------------------------------------------------- | -------------------------------------------------------- |
| `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `f32`, `f64` | `number`                                                 |
| `u64`, `i64`, `u128`, `i128`                         | `bigint`                                                 |
| `String`, `&str`, `char`                             | `string`                                                 |
| `bool`                                               | `boolean`                                                |
| `Option<T>`                                          | `T` (with `?` on field)                                  |
| `Vec<T>`, `[T; N]`                                   | `T[]`                                                    |
| `HashMap<K, V>`, `BTreeMap<K, V>`                    | `Record<K, V>`                                           |
| Tuples `(A, B, C)`                                   | `[A, B, C]`                                              |
| `Result<T, E>`                                       | `{ ok: T; err: undefined } \| { ok: undefined; err: E }` |

## 🗺️ Attributes

### Container Attributes

```rust
#[t0x(rename = "TypeName")]         // Rename the type
#[t0x(rename_all = "camelCase")]    // Rename all fields
#[t0x(tag = "type")]                // Internally tagged enum
#[t0x(tag = "t", content = "c")]    // Adjacently tagged enum
#[t0x(untagged)]                    // Untagged enum
```

### Field/Variant Attributes

```rust
#[t0x(rename = "fieldName")]        // Rename field
#[t0x(skip)]                        // Skip field
#[t0x(optional)]                    // Mark as optional (adds ?)
#[t0x(type = "CustomType")]         // Override TypeScript type
#[t0x(flatten)]                     // Flatten nested struct
```

## 🔁 Enum Representations

```rust
// Externally tagged (default)
#[derive(T0x)]
enum Message {
    Text(String),
    Number(i32),
}
// { Text: string } | { Number: number }

// Internally tagged
#[derive(T0x)]
#[t0x(tag = "type")]
enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: String },
}
// { type: "Click"; x: number; y: number } | { type: "KeyPress"; key: string }

// Untagged
#[derive(T0x)]
#[t0x(untagged)]
enum Value {
    Str(String),
    Num(i32),
}
// string | number
```
