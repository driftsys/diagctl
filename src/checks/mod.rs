use crate::svg::Svg;

pub mod aspect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub id: &'static str,
    pub layer: u8,
    pub status: Status,
    pub detail: String,
}

pub fn run_all(svg: &Svg) -> Vec<CheckResult> {
    vec![aspect::check(svg)]
}
