use crate::checks::{self, Status};
use crate::cli::Target;
use crate::output::{self, Error};
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

/// Checks each file in order. A file that cannot be read or parsed as SVG is
/// reported as an error and the other files are still checked. Exit code: 2
/// if any file could not be read or parsed, otherwise 1 if any check failed,
/// otherwise 0.
pub fn run(files: &[PathBuf], target: Target, json: bool) -> anyhow::Result<i32> {
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
                Status::Pass => "PASS",
                Status::Fail => "FAIL",
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
