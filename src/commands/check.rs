use crate::checks::{self, Status};
use crate::cli::Target;
use crate::svg;
use std::path::Path;

pub fn run(file: &Path, target: Target) -> anyhow::Result<i32> {
    let svg = svg::load(file)?;
    let results = checks::run_all(&svg, target);
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
