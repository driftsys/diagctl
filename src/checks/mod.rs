use crate::svg::Svg;

pub mod aspect;
pub mod edge_crossings;
pub mod node_overlap;
pub mod viewbox;

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
    vec![
        viewbox::check(svg),
        aspect::check(svg),
        node_overlap::check(svg),
        edge_crossings::check(svg),
    ]
}
