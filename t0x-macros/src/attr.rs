//! Attribute parsing for t0x derive macros

use syn::{Attribute, Lit, Result};

pub struct ContainerAttr {
    pub rename: Option<String>,
    pub as_name: Option<String>,
    pub rename_all: Option<RenameRule>,
    pub tag: Option<String>,
    pub content: Option<String>,
    pub untagged: bool,
}

impl Default for ContainerAttr {
    fn default() -> Self {
        Self {
            rename: None,
            as_name: None,
            rename_all: Some(RenameRule::CamelCase),
            tag: None,
            content: None,
            untagged: false,
        }
    }
}

#[derive(Default)]
pub struct FieldAttr {
    pub rename: Option<String>,
    pub skip: bool,
    pub exclude: bool,
    pub optional: bool,
    pub r#type: Option<String>,
    pub flatten: bool,
    pub docs: Vec<String>,
}

#[allow(clippy::enum_variant_names)] // Nobody cares
#[derive(Clone, Copy)]
pub enum RenameRule {
    LowerCase,
    UpperCase,
    CamelCase,
    SnakeCase,
    PascalCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    pub fn apply(&self, name: &str) -> String {
        match self {
            RenameRule::LowerCase => name.to_lowercase(),
            RenameRule::UpperCase => name.to_uppercase(),
            RenameRule::CamelCase => to_camel_case(name),
            RenameRule::SnakeCase => to_snake_case(name),
            RenameRule::PascalCase => to_pascal_case(name),
            RenameRule::ScreamingSnakeCase => to_snake_case(name).to_uppercase(),
            RenameRule::KebabCase => to_snake_case(name).replace('_', "-"),
            RenameRule::ScreamingKebabCase => to_snake_case(name).to_uppercase().replace('_', "-"),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(RenameRule::LowerCase),
            "UPPERCASE" => Some(RenameRule::UpperCase),
            "camelCase" => Some(RenameRule::CamelCase),
            "snake_case" => Some(RenameRule::SnakeCase),
            "PascalCase" => Some(RenameRule::PascalCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Some(RenameRule::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(RenameRule::ScreamingKebabCase),
            _ => None,
        }
    }
}

impl ContainerAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("t0x") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.rename = Some(s.value());
                    }
                } else if meta.path.is_ident("as_name") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.as_name = Some(s.value());
                    }
                } else if meta.path.is_ident("rename_all") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.rename_all = RenameRule::from_str(&s.value());
                    }
                } else if meta.path.is_ident("tag") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.tag = Some(s.value());
                    }
                } else if meta.path.is_ident("content") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.content = Some(s.value());
                    }
                } else if meta.path.is_ident("untagged") {
                    result.untagged = true;
                }
                Ok(())
            })?;
        }

        Ok(result)
    }
}

impl FieldAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta
                    && let syn::Expr::Lit(expr_lit) = &nv.value
                    && let Lit::Str(lit_str) = &expr_lit.lit
                {
                    result.docs.push(lit_str.value());
                }
                continue;
            }

            if !attr.path().is_ident("t0x") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.rename = Some(s.value());
                    }
                } else if meta.path.is_ident("skip") {
                    result.skip = true;
                } else if meta.path.is_ident("exclude") {
                    result.exclude = true;
                } else if meta.path.is_ident("optional") {
                    result.optional = true;
                } else if meta.path.is_ident("type") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result.r#type = Some(s.value());
                    }
                } else if meta.path.is_ident("flatten") {
                    result.flatten = true;
                }
                Ok(())
            })?;
        }

        Ok(result)
    }
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}
