use crate::output::{self, Error};
use serde_json::json;
use std::path::Path;

pub fn run(_file: &Path, json: bool) -> anyhow::Result<i32> {
    let errors = [Error {
        code: "not_implemented",
        message: "freshness: not yet implemented (tracking: https://github.com/driftsys/metapowers/issues/14)"
            .to_string(),
        file: None,
    }];
    Ok(output::finish(
        "freshness",
        json,
        json!([]),
        json!({}),
        &errors,
        "",
        false,
    ))
}
