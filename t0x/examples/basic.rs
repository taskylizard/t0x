//! Basic example of using t0x to generate TypeScript types

use t0x::T0x;

/// A user in the system
#[derive(T0x)]
struct User {
    /// The user's unique identifier
    id: u64,
    /// The user's display name
    name: String,
    /// The user's email address (optional)
    email: Option<String>,
    /// Whether the user is active
    active: bool,
}

/// Represents different types of messages
#[derive(T0x)]
#[t0x(tag = "type")]
enum Message {
    /// A simple text message
    Text { content: String },
    /// An image message
    Image {
        url: String,
        width: u32,
        height: u32,
    },
    /// A system notification
    System,
}

/// A paginated response
#[derive(T0x)]
struct PaginatedResponse {
    items: Vec<User>,
    total: u32,
    page: u32,
    per_page: u32,
}

fn main() {
    println!("// User type:");
    println!("{}", User::type_def());

    println!("// Message type:");
    println!("{}", Message::type_def());

    println!("// PaginatedResponse type:");
    println!("{}", PaginatedResponse::type_def());
}
