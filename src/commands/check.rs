use crate::checks::{self, Status};
use crate::cli::Target;
use crate::output::{self, paint, Error, GREEN, RED};
use crate::svg;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;

/// One check result for one file, as written in the JSON envelope.
#[derive(Serialize)]
struct Report {
    file: String,
    check: &'static str,
    layer: u8,
    status: Status,
    detail: String,
}

/// The last line of text output: "N checks: P passed, F failed", with
/// "1 check" in the singular.
fn summary_line(total: usize, failed: usize) -> String {
    let checks = if total == 1 {
        "1 check".to_string()
    } else {
        format!("{total} checks")
    };
    format!("{checks}: {} passed, {failed} failed\n", total - failed)
}

/// Checks each file in order. A file that cannot be read or parsed as SVG is
/// reported as an error and the other files are still checked. Exit code: 2
/// if any file could not be read or parsed, otherwise 1 if any check failed,
/// otherwise 0.
///
/// `_strict` turns REVIEW into a failure. No check reports REVIEW yet, so
/// the flag has no effect.
pub fn run(files: &[PathBuf], target: Target, json: bool, _strict: bool) -> anyhow::Result<i32> {
    let many = files.len() > 1;
    let mut reports = Vec::new();
    let mut errors = Vec::new();
    let mut text = String::new();
    for file in files {
        let name = file.display().to_string();
        let svg = match svg::load(file) {
            Ok(svg) => svg,
            Err(err) => {
                errors.push(Error {
                    code: err.code(),
                    message: format!("{name}: {err}"),
                    file: Some(name),
                });
                continue;
            }
        };
        if many {
            text.push_str(&format!("{name}\n"));
        }
        for r in checks::run_all(&svg, target) {
            let tag = match r.status {
                Status::Pass => paint("PASS", GREEN),
                Status::Fail => paint("FAIL", RED),
            };
            text.push_str(&format!("{tag}  [L{}] {}  {}\n", r.layer, r.id, r.detail));
            reports.push(Report {
                file: name.clone(),
                check: r.id,
                layer: r.layer,
                status: r.status,
                detail: r.detail,
            });
        }
    }
    let failed = reports.iter().filter(|r| r.status == Status::Fail).count();
    // No summary line when every file failed to load: "0 checks" would read
    // as a clean result.
    if errors.len() < files.len() {
        text.push_str(&summary_line(reports.len(), failed));
    }
    let summary = json!({
        "checks": reports.len(),
        "pass": reports.len() - failed,
        "fail": failed,
    });
    Ok(output::finish(
        "check",
        json,
        json!(reports),
        summary,
        &errors,
        &text,
        failed > 0,
    ))
}

#[cfg(test)]
mod tests {
    use super::summary_line;

    #[test]
    fn summary_line_uses_singular_for_one_check() {
        assert_eq!(summary_line(1, 0), "1 check: 1 passed, 0 failed\n");
        assert_eq!(summary_line(1, 1), "1 check: 0 passed, 1 failed\n");
        assert_eq!(summary_line(0, 0), "0 checks: 0 passed, 0 failed\n");
        assert_eq!(summary_line(22, 2), "22 checks: 20 passed, 2 failed\n");
    }
}
