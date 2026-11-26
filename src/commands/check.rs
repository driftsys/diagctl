use crate::checks::{self, Status};
use crate::cli::Target;
use crate::svg;
use std::path::PathBuf;

/// Checks each file in order. A file that cannot be read or parsed as SVG is
/// reported on stderr and the other files are still checked. Exit code: 2 if
/// any file could not be read or parsed, otherwise 1 if any check failed,
/// otherwise 0.
pub fn run(files: &[PathBuf], target: Target) -> anyhow::Result<i32> {
    let many = files.len() > 1;
    let mut any_failed = false;
    let mut any_error = false;
    for file in files {
        let svg = match svg::load(file) {
            Ok(svg) => svg,
            Err(err) => {
                eprintln!("diagctl: {}: {err:#}", file.display());
                any_error = true;
                continue;
            }
        };
        if many {
            println!("{}", file.display());
        }
        for r in checks::run_all(&svg, target) {
            let tag = match r.status {
                Status::Pass => "PASS",
                Status::Fail => {
                    any_failed = true;
                    "FAIL"
                }
            };
            println!("{tag}  [L{}] {}  {}", r.layer, r.id, r.detail);
        }
    }
    Ok(if any_error {
        2
    } else if any_failed {
        1
    } else {
        0
    })
}
