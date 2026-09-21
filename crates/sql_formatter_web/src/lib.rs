use sql_formatter_core::format::{FormatOptions, KeywordCase, SqlDialect};
use sql_formatter_core::{format_sql, minify_sql, validate_sql};
use wasm_bindgen::prelude::*;

/// Converts a CoreError into a JsValue error carrying a stable `code` field.
fn core_err(e: sql_formatter_core::CoreError) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"code".into(), &e.code().into()).ok();
    js_sys::Reflect::set(&obj, &"message".into(), &e.to_string().into()).ok();
    obj.into()
}

/// Format SQL with configurable options. Returns the formatted SQL string.
#[wasm_bindgen]
pub fn format(
    sql: &str,
    dialect: &str,
    indent_size: u8,
    keyword_case: &str,
    comma_before: bool,
    lines_between_statements: u8,
    max_line_width: u32,
) -> Result<String, JsValue> {
    let dialect = SqlDialect::from_str(dialect).map_err(core_err)?;
    let options = FormatOptions {
        dialect,
        indent_size,
        keyword_case: KeywordCase::from_str(keyword_case),
        comma_before,
        lines_between_statements,
        max_line_width,
    };
    format_sql(sql, &options).map_err(core_err)
}

/// Minify SQL to a single line. Returns the minified SQL string.
#[wasm_bindgen]
pub fn minify(sql: &str) -> Result<String, JsValue> {
    minify_sql(sql).map_err(core_err)
}

/// Validate SQL syntax. Returns a JSON string with `valid` and `issues` fields.
#[wasm_bindgen]
pub fn validate(sql: &str, dialect: &str) -> Result<String, JsValue> {
    let _ = SqlDialect::from_str(dialect).map_err(core_err)?;
    let issues = validate_sql(sql).map_err(core_err)?;
    let result = serde_json::json!({
        "valid": issues.is_empty(),
        "issues": issues,
    });
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization failed: {e}")))
}

/// Return the list of supported SQL dialects as a JSON array string.
#[wasm_bindgen]
pub fn list_dialects() -> String {
    let dialects = SqlDialect::list_all();
    serde_json::to_string(&dialects).unwrap_or_default()
}

/// Initialise panic hook for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
