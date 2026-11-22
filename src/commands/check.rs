use crate::checks::{self, Status};
use crate::cli::Target;
use crate::svg;
use std::path::Path;

// Both targets run the same checks for now. The markdown checks come with
// #19, #20 and #21.
pub fn run(file: &Path, _target: Target) -> anyhow::Result<i32> {
    let svg = svg::load(file)?;
    let results = checks::run_all(&svg);
    let mut any_failed = false;
    for r in &results {
        let tag = match r.status {
            Status::Pass => "PASS",
            Status::Fail => {
                any_failed = true;
                "FAIL"
            }
        };
        println!("{tag}  [L{}] {}  {}", r.layer, r.id, r.detail);
    }
    Ok(if any_failed { 1 } else { 0 })
}
