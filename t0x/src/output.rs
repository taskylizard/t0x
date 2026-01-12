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
    let stmt = Statement::from(decl);

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
                    result.push_str(&indent_str);
                    result.push_str(doc);
                    result.push('\n');
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

// TODO: A bit dogshit, should be more native
fn format_jsdoc(doc: &str) -> String {
    let lines: Vec<&str> = doc.lines().collect();
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
