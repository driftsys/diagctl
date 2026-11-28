//! Writes command output: text lines, or the JSON envelope of tblctl AD-0005.

use anstyle::{AnsiColor, Style};
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

/// The styles of the state words in text output.
pub const GREEN: Style = AnsiColor::Green.on_default();
pub const RED: Style = AnsiColor::Red.on_default();

/// Returns `word` in `style`. The ANSI codes are removed on write unless
/// stdout is a terminal or `CLICOLOR_FORCE` is set to a non-empty value; a
/// non-empty `NO_COLOR` removes them in all cases.
pub fn paint(word: &str, style: Style) -> String {
    format!("{style}{word}{style:#}")
}

/// Writes the output of `command` and returns the exit code: 2 if there is
/// any error or if writing stdout fails (other than a closed pipe), otherwise
/// 1 if `failed`, otherwise 0.
///
/// With `json`, stdout gets one pretty-printed object with `schema`,
/// `results`, `summary` and `errors`; otherwise stdout gets `text`. In both
/// modes each error is also written to stderr as one `diagctl: <message>`
/// line. All stderr lines are written before stdout, so in a merged stream
/// the errors come first.
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
    let mut stdout = anstream::AutoStream::auto(std::io::stdout().lock());
    match stdout.write_all(body.as_bytes()) {
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
