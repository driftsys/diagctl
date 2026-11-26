//! Writes command output: text lines, or the JSON envelope of tblctl AD-0005.

use serde::Serialize;
use serde_json::{json, Value};
use std::io::Write;

/// An operational error. `code` is stable; consumers read it instead of
/// `message`.
#[derive(Serialize)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// Writes the output of `command` and returns the exit code: 2 if there is
/// any error, otherwise 1 if `failed`, otherwise 0.
///
/// With `json`, stdout gets one pretty-printed object with `schema`,
/// `results`, `summary` and `errors`; otherwise stdout gets `text`. In both
/// modes each error is also written to stderr as one `diagctl: <message>`
/// line.
pub fn finish(
    command: &str,
    json: bool,
    results: Value,
    summary: Value,
    errors: &[Error],
    text: &str,
    failed: bool,
) -> i32 {
    for err in errors {
        // A closed stderr must not stop the envelope or change the exit code.
        let _ = writeln!(std::io::stderr(), "diagctl: {}", err.message);
    }
    let body = if json {
        let envelope = json!({
            "schema": format!("diagctl/{command}/v1"),
            "results": results,
            "summary": summary,
            "errors": errors,
        });
        serde_json::to_string_pretty(&envelope).expect("a JSON value always serializes") + "\n"
    } else {
        text.to_string()
    };
    // A closed stdout (for example `| head`) is not an error: the exit code
    // still reports the result.
    match std::io::stdout().lock().write_all(body.as_bytes()) {
        Err(err) if err.kind() != std::io::ErrorKind::BrokenPipe => {
            let _ = writeln!(std::io::stderr(), "diagctl: {err}");
            return 2;
        }
        _ => {}
    }
    if !errors.is_empty() {
        2
    } else if failed {
        1
    } else {
        0
    }
}
