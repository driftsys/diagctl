use crate::ascii::pipeline;
use std::io::Read;

/// Read pipeline labels (one per line) from stdin and emit an aligned ASCII
/// box-and-arrow diagram on stdout.
pub fn run() -> anyhow::Result<i32> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let labels: Vec<String> = input
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    if labels.is_empty() {
        eprintln!("ascii: no labels on stdin (expected one pipeline label per line)");
        return Ok(2);
    }
    println!("{}", pipeline::render(&labels));
    Ok(0)
}
