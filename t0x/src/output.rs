//! Code generation utilities for producing TypeScript output

use oxc_allocator::{Allocator, Vec as OxcVec};
use oxc_ast::{AstBuilder, ast::*};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_span::{SPAN, SourceType};

pub struct TypeOutput<F>
where
    F: for<'a> Fn(AstBuilder<'a>) -> TSType<'a>,
{
    pub name: &'static str,
    pub docs: Option<&'static str>,
    pub build_type: F,
}

pub fn generate_ts_code<F>(allocator: &Allocator, output: &TypeOutput<F>) -> String
where
    F: for<'a> Fn(AstBuilder<'a>) -> TSType<'a>,
{
    let ast = AstBuilder::new(allocator);

    let id = ast.binding_identifier(SPAN, output.name);
    let type_annotation = (output.build_type)(ast);

    let type_alias = ast.ts_type_alias_declaration(
        SPAN,
        id,
        None::<oxc_allocator::Box<'_, TSTypeParameterDeclaration<'_>>>,
        type_annotation,
        false,
    );

    let decl = Declaration::TSTypeAliasDeclaration(ast.alloc(type_alias));
    let export_decl = ast.export_named_declaration(
        SPAN,
        Some(decl),
        OxcVec::new_in(allocator),
        None::<StringLiteral<'_>>,
        ImportOrExportKind::Type,
        None::<WithClause<'_>>,
    );
    let stmt = Statement::ExportNamedDeclaration(ast.alloc(export_decl));

    let mut body = OxcVec::new_in(allocator);
    body.push(stmt);

    let program = ast.program(
        SPAN,
        SourceType::ts(),
        "",
        OxcVec::new_in(allocator),
        None,
        OxcVec::new_in(allocator),
        body,
    );

    let options = CodegenOptions::default();
    let result = Codegen::new().with_options(options).build(&program);

    let field_docs = crate::__private::take_field_docs();

    let mut output_str = String::new();

    if let Some(docs) = output.docs {
        output_str.push_str(&format_jsdoc(docs));
    }

    let code = inject_field_docs(&result.code, &field_docs);
    output_str.push_str(&code);
    output_str
}

fn inject_field_docs(code: &str, field_docs: &std::collections::HashMap<String, String>) -> String {
    if field_docs.is_empty() {
        return code.to_string();
    }

    let mut result = String::with_capacity(code.len() + field_docs.len() * 50);

    for line in code.lines() {
        let trimmed = line.trim();

        let mut doc_inserted = false;
        for (field_name, doc) in field_docs {
            let patterns = [format!("{field_name}:"), format!("{field_name}?:")];

            for pattern in &patterns {
                if trimmed.starts_with(pattern.as_str()) {
                    let indent = line.len() - line.trim_start().len();
                    let indent_str: String = line.chars().take(indent).collect();
                    push_indented_doc(&mut result, &indent_str, doc);
                    doc_inserted = true;
                    break;
                }
            }
            if doc_inserted {
                break;
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    if result.ends_with('\n') && !code.ends_with('\n') {
        result.pop();
    }

    result
}

fn push_indented_doc(result: &mut String, indent: &str, doc: &str) {
    for line in doc.lines() {
        result.push_str(indent);
        result.push_str(line);
        result.push('\n');
    }
}

pub(crate) fn format_jsdoc(doc: &str) -> String {
    let converted = rustdoc_links_to_jsdoc(doc);
    let lines: Vec<&str> = converted.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    if lines.len() == 1 {
        return format!("/** {} */\n", lines[0].trim());
    }

    let mut result = String::from("/**\n");
    for line in lines {
        result.push_str(" * ");
        result.push_str(line.trim());
        result.push('\n');
    }
    result.push_str(" */\n");
    result
}

fn rustdoc_links_to_jsdoc(doc: &str) -> String {
    let mut result = String::with_capacity(doc.len());
    let mut last = 0;

    for (start, ch) in doc.char_indices() {
        if start < last {
            continue;
        }

        if ch != '[' || is_escaped(doc, start) || is_image_link(doc, start) {
            continue;
        }

        let Some(label_end) = find_closing(doc, start, '[', ']') else {
            continue;
        };
        let label = &doc[start + 1..label_end];
        let after_label = label_end + 1;

        if doc[after_label..].starts_with("[]") {
            if let Some(link) = jsdoc_link(label, label, false) {
                result.push_str(&doc[last..start]);
                result.push_str(&link);
                last = after_label + 2;
            }
        } else if doc[after_label..].starts_with('(') {
            let target_start = after_label + 1;
            if let Some(target_end) = find_closing(doc, after_label, '(', ')') {
                let target = &doc[target_start..target_end];
                if let Some(link) = jsdoc_link(label, target, true) {
                    result.push_str(&doc[last..start]);
                    result.push_str(&link);
                    last = target_end + 1;
                }
            }
        } else if doc[after_label..].starts_with('[') {
            let target_start = after_label + 1;
            if let Some(target_end) = find_closing(doc, after_label, '[', ']') {
                let target = &doc[target_start..target_end];
                if let Some(link) = jsdoc_link(label, target, false) {
                    result.push_str(&doc[last..start]);
                    result.push_str(&link);
                    last = target_end + 1;
                }
            }
        } else if let Some(link) = jsdoc_link(label, label, false) {
            result.push_str(&doc[last..start]);
            result.push_str(&link);
            last = after_label;
        }
    }

    if last == 0 {
        doc.to_string()
    } else {
        result.push_str(&doc[last..]);
        result
    }
}

fn jsdoc_link(label: &str, target: &str, allow_url: bool) -> Option<String> {
    let label = strip_code_ticks(label.trim());
    let target = strip_code_ticks(target.trim());
    if label.is_empty() || target.is_empty() || label.contains('\n') || target.contains('\n') {
        return None;
    }

    let target = if allow_url && is_url(target) {
        target.to_string()
    } else {
        rust_path_to_jsdoc_target(target)?
    };

    if label == target {
        Some(format!("{{@link {target}}}"))
    } else {
        Some(format!("{{@link {target} {label}}}"))
    }
}

fn rust_path_to_jsdoc_target(path: &str) -> Option<String> {
    let path = path
        .strip_prefix("crate::")
        .or_else(|| path.strip_prefix("self::"))
        .or_else(|| path.strip_prefix("super::"))
        .unwrap_or(path);

    if path.is_empty()
        || !path
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | ':' | '.'))
    {
        return None;
    }

    let mut target = path.replace("::", ".");
    if let Some(stripped) = target.strip_prefix('.') {
        target = stripped.to_string();
    }

    if target.is_empty() {
        None
    } else {
        Some(target)
    }
}

fn strip_code_ticks(value: &str) -> &str {
    value
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or(value)
}

fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn is_escaped(doc: &str, start: usize) -> bool {
    let mut slash_count = 0;
    for ch in doc[..start].chars().rev() {
        if ch != '\\' {
            break;
        }
        slash_count += 1;
    }
    slash_count % 2 == 1
}

fn is_image_link(doc: &str, start: usize) -> bool {
    doc[..start].ends_with('!')
}

fn find_closing(doc: &str, open_at: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0;
    for (idx, ch) in doc[open_at..].char_indices() {
        let absolute = open_at + idx;
        if is_escaped(doc, absolute) {
            continue;
        }

        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(absolute);
            }
        }
    }

    None
}
